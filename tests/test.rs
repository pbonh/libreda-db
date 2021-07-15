
// use std::fs::File;
// use std::io::Read;

// /// Read a whole file into a byte vector.
// fn read_file(path: &str) -> Vec<u8> {
//     let mut f = File::open(path).unwrap();
//     let mut data = Vec::new();
//     f.read_to_end(&mut data).unwrap();
//     data
// }

// GDS tests
// #[test]
// fn test_load_inverter_cell() {
//     let f = File::open("./tests/data/INVX1.gds").unwrap();
//     let mut reader = BufReader::new(f);
//     let library = gds::Library::read(&mut reader).unwrap();
//     dbg!(&library);
//     println!("{}", library);
//     dbg!(library.name);
//
//     for st in library.structures {
//         dbg!(st.name);
//         for el in st.elements {
//             dbg!(el);
//         }
//     }
// }
//
// #[test]
// fn test_load_gds() {
//     let f = File::open("./tests/data/shapes_and_text.gds").unwrap();
//     let mut reader = BufReader::new(f);
//     let library = gds::Library::read(&mut reader).unwrap();
//     dbg!(&library);
//     println!("{}", library);
//     dbg!(library.name);
//
//     for st in library.structures {
//         dbg!(st.name);
//         for el in st.elements {
//             dbg!(el);
//         }
//     }
// }
//
