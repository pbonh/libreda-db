/*
 * Copyright (c) 2020-2021 Thomas Kramer.
 *
 * This file is part of LibrEDA
 * (see https://codeberg.org/libreda).
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

//! Collection of useful algorithms for layout processing.

use crate::prelude::{Rect, REdge};
use itertools::Itertools;
use num_traits::{Bounded, Zero};

/// Decompose a set of manhattanized polygons into non-overlapping horizontal rectangles.
/// The polygons in form of an iterator over all the edges.
/// A point is considered inside the polygon when the polygon wraps around it a non-zero number of times.
///
/// # Example
/// ```
///     use libreda_db::prelude::*;
///     use libreda_db::layout::algorithms::decompose_rectangles;
///     // Use a polygon like the following.
///     //    +--+
///     //    |  |
///     // +--+  |
///     // |     |
///     // +-----+
///     let poly = SimpleRPolygon::try_new(vec![
///         (0, 0), (2, 0), (2, 2), (1, 2), (1, 1), (0, 1)
///     ].iter().map(|t| Point::from(t)).collect()).unwrap();
///
///     // Decompose the polygon into non-overlapping horizontal rectangles.
///     let rects = decompose_rectangles(poly.edges());
///     assert_eq!(rects, vec![Rect::new((0, 0), (2, 1)), Rect::new((1, 1), (2, 2))]);
/// ```
pub fn decompose_rectangles<T, I>(redges: I) -> Vec<Rect<T>>
    where T: Ord + Bounded + Copy + Zero,
          I: Iterator<Item=REdge<T>> {
    // Sweep through the vertical edges from left to right. Keep track of 'open' rectangles.
    // A rectangle is opened when a left boundary is encountered and closed when a right boundary is encountered.
    // Construct a rectangle once a right boundary is encountered.

    {
        // Extract the vertical edges.
        let vertical_edges: Vec<_> = redges
            .filter(|e| e.is_vertical() && e.start != e.end)
            .map(|e| {
                let is_left = e.start > e.end;
                if is_left {
                    (e.reversed(), true)
                } else {
                    (e, false)
                }
            })
            // Sort edges from left to right, then by the y-coordinate of start and end point.
            .sorted_by_key(|(e, _is_left)| (e.offset, e.start, e.end))
            .collect();

        // Store the open rectangle as a tuple (y-start, y-end, inside count, x-position of opening vertical edge).
        let mut open_rects: Vec<(T, T, isize, T)> = vec![(T::min_value(), T::max_value(), 0, T::min_value())];
        let mut results = Vec::new();

        // Return the position of the new entry.
        fn split_intervals<T: PartialOrd + Copy>(intervals: &mut Vec<(T, T, isize, T)>, split_location: T) -> usize {
            // Find the interval which contains the split location.
            let (pos, &(a, b, value, x)) = intervals.iter()
                .enumerate()
                .find(|(_pos, &(a, b, _val, _x))| a <= split_location && split_location < b)
                .expect("Intervals must span the whole value range without gap.");

            if split_location == a || split_location == b {
                // Split location is equal to an end-point of the interval. No need to split there.
                pos
            } else {
                // Split location is inside the interval. Split the interval.
                let i1 = (a, split_location, value, x);
                let i2 = (split_location, b, value, x);
                intervals[pos] = i2;
                intervals.insert(pos, i1);
                pos + 1
            }
        }

        // Merge neighbouring intervals with the same value.
        fn merge_intervals<T: PartialEq + Copy>(intervals: &mut Vec<(T, T, isize, T)>) {
            debug_assert!(intervals.len() >= 1);
            let mut write_index = 0;
            let mut value = (intervals[0].2, intervals[0].3);
            for i in 1..intervals.len() {
                let (start, end, count, x) = intervals[i];
                let current_value = (count, x);
                intervals[write_index].1 = end;

                if current_value != value {
                    intervals[write_index].1 = start;
                    write_index += 1;
                    intervals[write_index] = intervals[i];
                    value = current_value;
                } else {
                    // Merge the intervals.
                }
            }

            // Crop the vector to the right size.
            intervals.truncate(write_index + 1);
        }

        // Process all vertical edges.
        for (current_edge, is_left) in vertical_edges {
            debug_assert!(current_edge.start < current_edge.end);

            {
                let pos = split_intervals(&mut open_rects, current_edge.start);

                let (_, _, value, _) = open_rects[pos];
                if value == 0 {
                    // This opens a new rectangle. Store the x-coordinate of the left edge.
                    open_rects[pos].3 = current_edge.offset;
                }

                split_intervals(&mut open_rects, current_edge.end);
            }

            let increment = if is_left {
                // Enter rectangle.
                1
            } else {
                // Exit rectangle.
                -1
            };

            {
                // Find rectangles that are closed by this boundary.
                // Find this by computing the intersection of the y-interval of the right boundary
                // with all open intervals that are about to be closed (which have value = -increment).

                let increment_inv = -increment;
                let closed_rects = open_rects.iter()
                    .take_while(|(_a, b, _value, _x)| b <= &current_edge.end)
                    .filter(|(a, _b, value, _x)| a >= &current_edge.start && value == &increment_inv)
                    .map(|&(a, b, _value, x_start)| {
                        // Compute the intersection of the intervals [a, b] and [e.start, e.end].
                        let y_start = a.max(current_edge.start);
                        let y_end = b.min(current_edge.end);
                        let x_end = current_edge.offset;
                        
                        Rect::new((x_start, y_start), (x_end, y_end))
                    });

                results.extend(closed_rects);
            }

            // Update the inside-count for open rectangles that interact with the current edge.
            open_rects.iter_mut()
                .take_while(|(_a, b, _value, _x)| b <= &current_edge.end)
                .filter(|(a, _b, _value, _x)| a >= &current_edge.start)
                .for_each(|(_, _, count, x)| {
                    *count += increment;

                    if *count == 0 {
                        // Reset the x-coordinate of the left boundary (there's none now).
                        *x = T::min_value();
                    }
                });


            // Simplify the intervals.
            merge_intervals(&mut open_rects);
        }

        results
    }
}

#[test]
fn test_decompose_rectangles_trivial() {
    // Test trivial cases: Empty polygon and rectangle.
    use crate::prelude::Point;
    use crate::prelude::SimpleRPolygon;

    let empty: SimpleRPolygon<i32> = SimpleRPolygon::empty();
    let rects = decompose_rectangles(empty.edges());
    assert_eq!(rects, vec![]);

    // Test with reversed polygon.
    let rect = SimpleRPolygon::try_new(vec![
        (0, 0), (2, 0), (2, 2), (0, 2)
    ].iter().map(|t| Point::from(t)).collect()).unwrap();
    let rects = decompose_rectangles(rect.reversed().edges());
    assert_eq!(rects, vec![Rect::new((0, 0), (2, 2))]);
}

#[test]
fn test_decompose_rectangles() {
    use crate::prelude::Point;
    use crate::prelude::SimpleRPolygon;
    //    +--+
    //    |  |
    // +--+  |
    // |     |
    // +-----+
    let poly = SimpleRPolygon::try_new(vec![
        (0, 0), (2, 0), (2, 2), (1, 2), (1, 1), (0, 1)
    ].iter().map(|t| Point::from(t)).collect()).unwrap();
    let rects = decompose_rectangles(poly.edges());
    assert_eq!(rects, vec![Rect::new((0, 0), (2, 1)), Rect::new((1, 1), (2, 2))]);

    // Test with reversed polygon.
    let rects = decompose_rectangles(poly.reversed().edges());
    assert_eq!(rects, vec![Rect::new((0, 0), (2, 1)), Rect::new((1, 1), (2, 2))]);
}

#[test]
fn test_decompose_rectangles_overlapping() {
    use crate::prelude::Point;
    use crate::prelude::SimpleRPolygon;
    use crate::prelude::IntoEdges;

    let rects = vec![
        Rect::new((0i32, 0), (10, 10)),
        Rect::new((0, 0), (5, 5)),
    ];

    let decomposed = decompose_rectangles(rects.iter().flat_map(|r| r.into_edges()));
    assert_eq!(decomposed, vec![Rect::new((0, 0), (10, 10))]);
}