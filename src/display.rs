// use crate::{Atom, BondType, Model};
//
// enum Position {
//     LEFT,
//     RIGHT,
//     UP,
//     DOWN,
// }
//
// #[derive(Debug)]
// enum Geometry {
//     Linear2,
//     Linear,
//     TrigonalPlanar,
//     Bent1Lone,
//     Tetrahedral,
//     TrigonalPyramidal,
//     Bent2Lone,
//     TrigonalBipyramidal,
//     Seesaw,
//     TShape,
//     Linear3Lone,
//     Octahedral,
//     SquarePyramidal,
//     SquarePlanar,
//     PentagonalBipyramidal,
//     PentagonalPyramidal,
//     PentagonalPlanar,
//     SquareAntiprismatic
// }
//
// pub fn display(model: Model) {
//     let num_of_atoms = model.atoms.len();
//
//     // determine central atom
//     let mut index_of_atom_with_most_bonds = 0;
//     for i in 1..num_of_atoms {
//         if model.bonds_with.get(i).unwrap().len() > model.bonds_with.get(index_of_atom_with_most_bonds).unwrap().len() {
//             index_of_atom_with_most_bonds = i;
//         }
//     }
//     let central_atom = model.atoms.get(index_of_atom_with_most_bonds).unwrap().borrow();
//
//     let mut indices_of_non_central_atoms = vec![];
//     let mut atoms_without_central = vec![];
//
//     for i in 0..num_of_atoms {
//         if !(model.atoms[i].borrow().name == central_atom.name && model.atoms[i].borrow().id == central_atom.id) {
//             indices_of_non_central_atoms.push(i);
//             atoms_without_central.push(model.atoms[i].clone());
//         }
//     }
//
//     let geometry = electron_geometry(&model, &central_atom).unwrap();
//
//     println!("{:?}", geometry);
//
//     match geometry {
//         Geometry::Linear2 => {
//             let atom_right = atoms_without_central.remove(0).borrow().clone();
//
//             let lone_left = central_atom.lone;
//             let lone_right = atom_right.lone;
//
//             let bond = model.bonds_with[0][0].1;
//
//             let lines_left = atom_lines(lone_left, &central_atom.name, Position::LEFT);
//             let lines_right = atom_lines(lone_right, &atom_right.name, Position::RIGHT);
//
//             for i in 0..3 {
//                 for j in 0..3 {
//                     if j == 0 {
//                         print!("{}", lines_left[i]);
//                     } else if j == 1 {
//                         if i == 1 {
//                             match bond {
//                                 BondType::SINGLE => { print!("--") }
//                                 BondType::DOUBLE => { print!("==") }
//                                 BondType::TRIPLE => { print!("≡≡") }
//                             }
//                         } else {
//                             print!("  ");
//                         }
//                     } else if j == 2 {
//                         print!("{}", lines_right[i]);
//                     }
//                 }
//                 println!();
//             }
//         }
//
//
//         Geometry::Linear => {
//             let atom_left  = atoms_without_central.remove(0).borrow().clone();
//             let atom_right = atoms_without_central.remove(0).borrow().clone();
//
//             let lone_left = atom_left.lone;
//             let lone_right = atom_right.lone;
//
//             let bond_left = model.bonds_with[indices_of_non_central_atoms[0]][0].1;
//             let bond_right = model.bonds_with[indices_of_non_central_atoms[1]][0].1;
//
//             let central_atom_line_2 = central_atom.clone().name;
//
//             let lines_left = atom_lines(lone_left, &atom_left.name, Position::LEFT);
//             let lines_right = atom_lines(lone_right, &atom_right.name, Position::RIGHT);
//
//             for i in 0..3 { // each line
//                 for j in 0..5 { // each block (left atom, bond, middle, bond, right)
//                     if j == 0 {
//                         print!("{}", lines_left[i]);
//                     } else if j == 1 {
//                         if i == 1 {
//                             match bond_left {
//                                 BondType::SINGLE => { print!("--") }
//                                 BondType::DOUBLE => { print!("==") }
//                                 BondType::TRIPLE => { print!("≡≡") }
//                             }
//                         } else {
//                             print!("  ");
//                         }
//
//                     } else if j == 2 {
//                         if i == 1 {
//                             print!("{}", central_atom_line_2);
//                         } else {
//                             print!("  ");
//                         }
//                     } else if j == 3 {
//                         if i == 1 {
//                             match bond_right {
//                                 BondType::SINGLE => { print!("--") }
//                                 BondType::DOUBLE => { print!("==") }
//                                 BondType::TRIPLE => { print!("≡≡") }
//                             }
//                         } else {
//                             print!("  ");
//                         }
//
//                     } else if j == 4 {
//                         print!("{}", lines_right[i]);
//                     }
//                 }
//                 println!();
//             }
//         }
//
//         Geometry::TrigonalPlanar => {
//             let atom_left  = atoms_without_central.remove(0).borrow().clone();
//             let atom_right = atoms_without_central.remove(0).borrow().clone();
//             let atom_top = atoms_without_central.remove(0).borrow().clone();
//
//             let lone_left = atom_left.lone;
//             let lone_right = atom_right.lone;
//             let lone_top = atom_top.lone;
//
//             let bond_left = model.bonds_with[indices_of_non_central_atoms[0]][0].1;
//             let bond_right = model.bonds_with[indices_of_non_central_atoms[1]][0].1;
//             let bond_top = model.bonds_with[indices_of_non_central_atoms[2]][0].1;
//
//             let central_atom_line = central_atom.name.clone();
//
//             let lines_left = atom_lines(lone_left, &atom_left.name, Position::DOWN);
//             let lines_right = atom_lines(lone_right, &atom_right.name, Position::DOWN);
//             let lines_top = atom_lines(lone_top, &atom_top.name, Position::UP);
//
//             //      ..
//             //     :XX:
//             //      |
//             //      XX
//             //    /    \
//             //  :XX:  :XX:
//             //   ..    ..
//
//             for i in 0..4 { // each block
//                 for j in 0..5 { // each position: left, middle, right
//                     if i == 0 {
//                         if j == 1 {
//                             print!(" {}", lines_top[0]);
//                             println!();
//                             print!("   {}", lines_top[1]);
//                         } else {
//                             print!("  ");
//                         }
//                     } else if i == 1 {
//                         if j == 2 {
//                             match bond_top {
//                                 BondType::SINGLE => { print!("| ") }
//                                 BondType::DOUBLE => { print!("||") }
//                                 _ => {}
//                             }
//                         } else {
//                             print!("  ");
//                         }
//                     } else if i == 2 {
//                         if j == 2 {
//                             print!("{}", central_atom_line);
//                         } else {
//                             print!("  ");
//                         }
//                     } else if i == 3 {
//                         if j == 1 {
//                             match bond_left {
//                                 BondType::SINGLE => { print!("/ ") }
//                                 BondType::DOUBLE => { print!("//") }
//                                 _ => {}
//                             }
//                         } else if j == 3 {
//                             match bond_right {
//                                 BondType::SINGLE => { print!("\\ ") }
//                                 BondType::DOUBLE => { print!("\\\\") }
//                                 _ => {}
//                             }
//                         } else {
//                             print!("  ");
//                         }
//                     }
//                 }
//                 println!();
//
//             }
//             println!("{}  {}", lines_left[0], lines_right[0]);
//             println!("{}  {}", lines_left[1], lines_right[1]);
//         }
//
//         Geometry::Bent1Lone | Geometry::Bent2Lone => {
//             let atom_left  = atoms_without_central.remove(0).borrow().clone();
//             let atom_right = atoms_without_central.remove(0).borrow().clone();
//
//             let lone_left = atom_left.lone;
//             let lone_right = atom_right.lone;
//
//             let bond_left = model.bonds_with[indices_of_non_central_atoms[0]][0].1;
//             let bond_right = model.bonds_with[indices_of_non_central_atoms[1]][0].1;
//
//             let central_atom_line = central_atom.name.clone();
//
//             let lines_left = atom_lines(lone_left, &atom_left.name, Position::DOWN);
//             let lines_right = atom_lines(lone_right, &atom_right.name, Position::DOWN);
//
//             //
//             //
//             //      ..
//             //      XX
//             //    /    \
//             //  :XX:  :XX:
//             //   ..    ..
//
//             for i in 0..2 { // each block vertically
//                 for j in 0..5 { // each block of two spaces horizontally
//                     if i == 0 {
//                         if j == 2 {
//                             print!("..");
//                             println!();
//                             match geometry {
//                                 Geometry::Bent2Lone => { print!("   ˙{}˙", central_atom_line); }
//                                 Geometry::Bent1Lone => { print!("    {}", central_atom_line); }
//                                 _ => {}
//                             }
//                         } else {
//                             print!("  ");
//                         }
//                     } else if i == 1 {
//                         if j == 1 {
//                             match bond_left {
//                                 BondType::SINGLE => { print!("/ ") }
//                                 BondType::DOUBLE => { print!("//") }
//                                 _ => {}
//                             }
//                         } else if j == 3 {
//                             match bond_right {
//                                 BondType::SINGLE => { print!("\\ ") }
//                                 BondType::DOUBLE => { print!("\\\\") }
//                                 _ => {}
//                             }
//                         } else {
//                             print!("  ");
//                         }
//                     }
//                 }
//                 println!();
//
//             }
//             println!("{}  {}", lines_left[0], lines_right[0]);
//             println!("{}  {}", lines_left[1], lines_right[1]);
//         }
//
//         Geometry::Tetrahedral => {
//             let atom_left  = atoms_without_central.remove(0).borrow().clone();
//             let atom_right = atoms_without_central.remove(0).borrow().clone();
//             let atom_up = atoms_without_central.remove(0).borrow().clone();
//             let atom_down = atoms_without_central.remove(0).borrow().clone();
//
//             let lone_left = atom_left.lone;
//             let lone_right = atom_right.lone;
//             let lone_up = atom_up.lone;
//             let lone_down = atom_down.lone;
//
//             let bond_left = model.bonds_with[indices_of_non_central_atoms[0]][0].1;
//             let bond_right = model.bonds_with[indices_of_non_central_atoms[1]][0].1;
//             let bond_up = model.bonds_with[indices_of_non_central_atoms[2]][0].1;
//             let bond_down = model.bonds_with[indices_of_non_central_atoms[3]][0].1;
//
//             let central_atom_line = central_atom.name.clone();
//
//             let lines_left = atom_lines(lone_left, &atom_left.name, Position::LEFT);
//             let lines_right = atom_lines(lone_right, &atom_right.name, Position::RIGHT);
//             let lines_up = atom_lines(lone_up, &atom_up.name, Position::UP);
//             let lines_down = atom_lines(lone_down, &atom_down.name, Position::DOWN);
//
//             //      ..
//             //     :XX:
//             //  ..  |   ..
//             // :XX--XX--XX:
//             //  ..  |   ..
//             //     :XX:
//             //      ..
//
//             for i in 0..5 { // 5 blocks vertically
//                 for j in 0..5 { // 6 blocks horizontally
//                     if i == 0 {
//                         if j == 2 {
//                             print!("{}", lines_up[0]);
//                             println!();
//                             print!("    {}", lines_up[1]);
//                         } else {
//                             print!("  ");
//                         }
//                     } else if i == 1 {
//                         if j == 0 {
//                             print!("{}", lines_left[0]);
//                         } else if j == 2 {
//                             match bond_up {
//                                 BondType::SINGLE => { print!("| ") }
//                                 BondType::DOUBLE => { print!("||") }
//                                 BondType::TRIPLE => {}
//                             }
//                         } else if j == 4 {
//                             print!("{}", lines_right[0]);
//                         } else {
//                             print!("  ");
//                         }
//                     } else if i == 2 {
//                         if j == 0 {
//                             print!("{}", lines_left[1]);
//                         } else if j == 1 {
//                             match bond_left {
//                                 BondType::SINGLE => { print!("--") }
//                                 BondType::DOUBLE => { print!("==") }
//                                 BondType::TRIPLE => { print!("≡≡") }
//                             }
//                         } else if j == 2 {
//                             print!("{}", central_atom_line);
//                         } else if j == 3 {
//                             match bond_right {
//                                 BondType::SINGLE => { print!("--") }
//                                 BondType::DOUBLE => { print!("==") }
//                                 BondType::TRIPLE => { print!("≡≡") }
//                             }
//                         } else if j == 4 {
//                             print!("{}", lines_right[1]);
//                         }
//                     } else if i == 3 {
//                         if j == 0 {
//                             print!("{}", lines_left[2]);
//                         } else if j == 2 {
//                             match bond_down {
//                                 BondType::SINGLE => { print!("| ") }
//                                 BondType::DOUBLE => { print!("||") }
//                                 BondType::TRIPLE => {}
//                             }
//                         } else if j == 4 {
//                             print!("{}", lines_right[2]);
//                         } else {
//                             print!("  ");
//                         }
//                     } else if i == 4 {
//                         if j == 2 {
//                             print!("{}", lines_down[0]);
//                             println!();
//                             print!("    {}", lines_down[1]);
//                         } else {
//                             print!("  ");
//                         }
//                     }
//                 }
//                 println!();
//             }
//         }
//
//         Geometry::TrigonalPyramidal => {
//             let atom_left  = atoms_without_central.remove(0).borrow().clone();
//             let atom_right = atoms_without_central.remove(0).borrow().clone();
//             let atom_down = atoms_without_central.remove(0).borrow().clone();
//
//             let lone_left = atom_left.lone;
//             let lone_right = atom_right.lone;
//             let lone_down = atom_down.lone;
//
//             let bond_left = model.bonds_with[indices_of_non_central_atoms[0]][0].1;
//             let bond_right = model.bonds_with[indices_of_non_central_atoms[1]][0].1;
//             let bond_down = model.bonds_with[indices_of_non_central_atoms[2]][0].1;
//
//             let central_atom_line = central_atom.name.clone();
//
//             let lines_left = atom_lines(lone_left, &atom_left.name, Position::LEFT);
//             let lines_right = atom_lines(lone_right, &atom_right.name, Position::RIGHT);
//             let lines_down = atom_lines(lone_down, &atom_down.name, Position::DOWN);
//
//             //
//             //
//             //  ..  ..  ..
//             // :XX--XX--XX:
//             //  ..  |   ..
//             //     :XX:
//             //      ..
//
//             for i in 0..4 {
//                 for j in 0..5 {
//                     if i == 0 {
//                         if j == 0 {
//                             print!("{}", lines_left[0]);
//                         } else if j == 2 {
//                             print!("..");
//                         } else if j == 4 {
//                             print!("{}", lines_right[0]);
//                         } else {
//                             print!("  ");
//                         }
//                     } else if i == 1 {
//                         if j == 0 {
//                             print!("{}", lines_left[1]);
//                         } else if j == 1 {
//                             match bond_left {
//                                 BondType::SINGLE => { print!("--") }
//                                 BondType::DOUBLE => { print!("==") }
//                                 BondType::TRIPLE => { print!("≡≡") }
//                             }
//                         } else if j == 2 {
//                             print!("{}", central_atom_line);
//                         } else if j == 3 {
//                             match bond_right {
//                                 BondType::SINGLE => { print!("--") }
//                                 BondType::DOUBLE => { print!("==") }
//                                 BondType::TRIPLE => { print!("≡≡") }
//                             }
//                         } else if j == 4 {
//                             print!("{}", lines_right[1]);
//                         }
//                     } else if i == 2 {
//                         if j == 0 {
//                             print!("{}", lines_left[2]);
//                         } else if j == 2 {
//                             match bond_down {
//                                 BondType::SINGLE => { print!("| ") }
//                                 BondType::DOUBLE => { print!("||") }
//                                 BondType::TRIPLE => {}
//                             }
//                         } else if j == 4 {
//                             print!("{}", lines_right[2]);
//                         } else {
//                             print!("  ");
//                         }
//                     } else if i == 3 {
//                         if j == 2 {
//                             print!("{}", lines_down[0]);
//                             println!();
//                             print!("    {}", lines_down[1]);
//                         } else {
//                             print!("  ");
//                         }
//                     }
//                 }
//                 println!();
//             }
//         }
//
//         Geometry::TrigonalBipyramidal => {
//             let atom_left  = atoms_without_central.remove(0).borrow().clone();
//             let atom_right = atoms_without_central.remove(0).borrow().clone();
//             let atom_down = atoms_without_central.remove(0).borrow().clone();
//             let atom_up_left = atoms_without_central.remove(0).borrow().clone();
//             let atom_up_right = atoms_without_central.remove(0).borrow().clone();
//
//             let lone_left = atom_left.lone;
//             let lone_right = atom_right.lone;
//             let lone_down = atom_down.lone;
//             let lone_up_left = atom_up_left.lone;
//             let lone_up_right = atom_up_right.lone;
//
//             let bond_left = model.bonds_with[indices_of_non_central_atoms[0]][0].1;
//             let bond_right = model.bonds_with[indices_of_non_central_atoms[1]][0].1;
//             let bond_down = model.bonds_with[indices_of_non_central_atoms[2]][0].1;
//             let bond_up_left = model.bonds_with[indices_of_non_central_atoms[3]][0].1;
//             let bond_up_right = model.bonds_with[indices_of_non_central_atoms[4]][0].1;
//
//             let central_atom_line = central_atom.name.clone();
//
//             let lines_left = atom_lines(lone_left, &atom_left.name, Position::LEFT);
//             let lines_right = atom_lines(lone_right, &atom_right.name, Position::RIGHT);
//             let lines_down = atom_lines(lone_down, &atom_down.name, Position::DOWN);
//             let lines_up_left = atom_lines(lone_up_left, &atom_up_left.name, Position::UP);
//             let lines_up_right = atom_lines(lone_up_right, &atom_up_right.name, Position::UP);
//
//             //   ..    ..
//             //  :XX:  :XX:
//             //    \\  //
//             //  .. \\// ..
//             // :XX--XX--XX:
//             //  ..  |   ..
//             //     :XX:
//             //      ..
//
//             for i in 0..6 {
//                 for j in 0..5 {
//                     if i == 0 {
//                         if j == 0 {
//                             print!(" {}", lines_up_left[0]);
//                         } else if j == 2 {
//                             print!("{}", lines_up_right[0]);
//                         } else {
//                             print!("  ");
//                         }
//                     } else if i == 1 {
//                         if j == 0 {
//                             print!(" {}", lines_up_left[1]);
//                         } else if j == 2 {
//                             print!("{}", lines_up_right[1]);
//                         } else {
//                             print!("  ");
//                         }
//                     } else if i == 2 {
//                         if j == 0 {
//                             match bond_up_left {
//                                 BondType::SINGLE => { print!("   \\ "); }
//                                 BondType::DOUBLE => { print!("   \\\\"); }
//                                 BondType::TRIPLE => {}
//                             }
//                             match bond_up_right {
//                                 BondType::SINGLE => { print!("  / "); }
//                                 BondType::DOUBLE => { print!("  //"); }
//                                 BondType::TRIPLE => {}
//                             }
//                         }
//                     } else if i == 3 {
//                         if j == 0 {
//                             print!("{}", lines_left[0]);
//                         } else if j == 1 {
//                             match bond_up_left {
//                                 BondType::SINGLE => { print!(" \\ "); }
//                                 BondType::DOUBLE => { print!(" \\\\"); }
//                                 BondType::TRIPLE => {}
//                             }
//                         } else if j == 2 {
//                             match bond_up_right {
//                                 BondType::SINGLE => { print!("/  "); }
//                                 BondType::DOUBLE => { print!("// "); }
//                                 BondType::TRIPLE => {}
//                             }
//                         } else if j == 3 {
//                             print!("{}", lines_right[0]);
//                         }
//                     } else if i == 4 {
//                         if j == 0 {
//                             print!("{}", lines_left[1]);
//                             match bond_left {
//                                 BondType::SINGLE => { print!("--"); }
//                                 BondType::DOUBLE => { print!("=="); }
//                                 BondType::TRIPLE => { print!("≡≡"); }
//                             }
//                             print!("{}",central_atom_line);
//                             match bond_right {
//                                 BondType::SINGLE => { print!("--"); }
//                                 BondType::DOUBLE => { print!("=="); }
//                                 BondType::TRIPLE => { print!("≡≡"); }
//                             }
//                             print!("{}", lines_right[1]);
//                         }
//                     } else if i == 5 {
//                         if j == 0 {
//                             print!("{}", lines_left[2]);
//                         } else if j == 2 {
//                             match bond_down {
//                                 BondType::SINGLE => { print!("| "); }
//                                 BondType::DOUBLE => { print!("||"); }
//                                 BondType::TRIPLE => {}
//                             }
//                         } else if j == 4 {
//                             print!("{}", lines_right[2]);
//                         } else {
//                             print!("  ");
//                         }
//                     }
//                 }
//                 println!();
//             }
//
//             println!("    {}", lines_down[0]);
//             println!("    {}", lines_down[1]);
//         }
//
//         Geometry::Seesaw => {
//             let atom_left  = atoms_without_central.remove(0).borrow().clone();
//             let atom_right = atoms_without_central.remove(0).borrow().clone();
//             let atom_down_left = atoms_without_central.remove(0).borrow().clone();
//             let atom_down_right = atoms_without_central.remove(0).borrow().clone();
//
//             let lone_left = atom_left.lone;
//             let lone_right = atom_right.lone;
//             let lone_down_left = atom_down_left.lone;
//             let lone_down_right = atom_down_right.lone;
//
//             let bond_left = model.bonds_with[indices_of_non_central_atoms[0]][0].1;
//             let bond_right = model.bonds_with[indices_of_non_central_atoms[1]][0].1;
//             let bond_down_left = model.bonds_with[indices_of_non_central_atoms[2]][0].1;
//             let bond_down_right = model.bonds_with[indices_of_non_central_atoms[3]][0].1;
//
//             let central_atom_line = central_atom.name.clone();
//
//             let lines_left = atom_lines(lone_left, &atom_left.name, Position::LEFT);
//             let lines_right = atom_lines(lone_right, &atom_right.name, Position::RIGHT);
//             let lines_down_left = atom_lines(lone_down_left, &atom_down_left.name, Position::DOWN);
//             let lines_down_right = atom_lines(lone_down_right, &atom_down_right.name, Position::DOWN);
//
//             //  ..  ..  ..
//             // :XX--XX--XX:
//             //  .. / \  ..
//             //    /   \
//             //  :XX:  :XX:
//             //   ..    ..
//
//             for i in 0..4 {
//                 for j in 0..5 {
//                     if i == 0 {
//                         if j == 0 {
//                             print!("{}", lines_left[0]);
//                         } else if j == 2 {
//                             print!("..");
//                         } else if j == 4 {
//                             print!("{}", lines_right[0]);
//                         } else {
//                             print!("  ");
//                         }
//                     } else if i == 1 {
//                         if j == 0 {
//                             print!("{}", lines_left[1]);
//                             match bond_left {
//                                 BondType::SINGLE => { print!("--"); }
//                                 BondType::DOUBLE => { print!("=="); }
//                                 BondType::TRIPLE => { print!("≡≡"); }
//                             }
//                             print!("{}",central_atom_line);
//                             match bond_right {
//                                 BondType::SINGLE => { print!("--"); }
//                                 BondType::DOUBLE => { print!("=="); }
//                                 BondType::TRIPLE => { print!("≡≡"); }
//                             }
//                             print!("{}", lines_right[1]);
//                         }
//                     } else if i == 2 {
//                         if j == 0 {
//                             print!("{}", lines_left[2]);
//                             match bond_down_left {
//                                 BondType::SINGLE => { print!(" / "); }
//                                 BondType::DOUBLE => { print!(" //"); }
//                                 BondType::TRIPLE => {}
//                             }
//                             match bond_down_right {
//                                 BondType::SINGLE => { print!("\\  "); }
//                                 BondType::DOUBLE => { print!("\\\\ "); }
//                                 BondType::TRIPLE => {}
//                             }
//                             print!("{}", lines_right[2]);
//                         }
//                     } else if i == 3 {
//                         if j == 0 {
//                             match bond_down_left {
//                                 BondType::SINGLE => { print!("   / "); }
//                                 BondType::DOUBLE => { print!("   //"); }
//                                 BondType::TRIPLE => {}
//                             }
//                             match bond_down_right {
//                                 BondType::SINGLE => { print!("  \\ "); }
//                                 BondType::DOUBLE => { print!("  \\\\"); }
//                                 BondType::TRIPLE => {}
//                             }
//                         }
//                     }
//                 }
//                 println!();
//             }
//
//             println!(" {}  {}", lines_down_left[0], lines_down_right[0]);
//             println!(" {}  {}", lines_down_left[1], lines_down_right[1]);
//         }
//
//         Geometry::TShape => {
//             let atom_left  = atoms_without_central.remove(0).borrow().clone();
//             let atom_right = atoms_without_central.remove(0).borrow().clone();
//             let atom_down = atoms_without_central.remove(0).borrow().clone();
//
//             let lone_left = atom_left.lone;
//             let lone_right = atom_right.lone;
//             let lone_down = atom_down.lone;
//
//             let bond_left = model.bonds_with[indices_of_non_central_atoms[0]][0].1;
//             let bond_right = model.bonds_with[indices_of_non_central_atoms[1]][0].1;
//             let bond_down = model.bonds_with[indices_of_non_central_atoms[2]][0].1;
//
//             let central_atom_line = central_atom.name.clone();
//
//             let lines_left = atom_lines(lone_left, &atom_left.name, Position::LEFT);
//             let lines_right = atom_lines(lone_right, &atom_right.name, Position::RIGHT);
//             let lines_down = atom_lines(lone_down, &atom_down.name, Position::DOWN);
//
//             //
//             //
//             //  ..  .. ..  ..
//             // :XX---XX ---XX:
//             //  ..   |     ..
//             //      :XX:
//             //       ..
//
//             for i in 0..4 {
//                 for j in 0..5 {
//                     if i == 0 {
//                         if j == 0 {
//                             print!("{}", lines_left[0]);
//                         } else if j == 2 {
//                             print!(".. ..");
//                         } else if j == 4 {
//                             print!("{}", lines_right[0]);
//                         } else {
//                             print!("  ");
//                         }
//                     } else if i == 1 {
//                         if j == 0 {
//                             print!("{}", lines_left[1]);
//                         } else if j == 1 {
//                             match bond_left {
//                                 BondType::SINGLE => { print!("---") }
//                                 BondType::DOUBLE => { print!("===") }
//                                 BondType::TRIPLE => { print!("≡≡≡") }
//                             }
//                         } else if j == 2 {
//                             print!("{} ", central_atom_line);
//                         } else if j == 3 {
//                             match bond_right {
//                                 BondType::SINGLE => { print!("---") }
//                                 BondType::DOUBLE => { print!("===") }
//                                 BondType::TRIPLE => { print!("≡≡≡") }
//                             }
//                         } else if j == 4 {
//                             print!("{}", lines_right[1]);
//                         }
//                     } else if i == 2 {
//                         if j == 0 {
//                             print!("{}", lines_left[2]);
//                         } else if j == 2 {
//                             match bond_down {
//                                 BondType::SINGLE => { print!(" | ") }
//                                 BondType::DOUBLE => { print!(" ||") }
//                                 BondType::TRIPLE => {}
//                             }
//                         } else if j == 4 {
//                             print!("  {}", lines_right[2]);
//                         } else {
//                             print!("  ");
//                         }
//                     } else if i == 3 {
//                         if j == 2 {
//                             print!(" {}", lines_down[0]);
//                             println!();
//                             print!("     {}", lines_down[1]);
//                         } else {
//                             print!("  ");
//                         }
//                     }
//                 }
//                 println!();
//             }
//         }
//
//         Geometry::Linear3Lone => {
//             let atom_left  = atoms_without_central.remove(0).borrow().clone();
//             let atom_right = atoms_without_central.remove(0).borrow().clone();
//
//             let lone_left = atom_left.lone;
//             let lone_right = atom_right.lone;
//
//             let bond_left = model.bonds_with[indices_of_non_central_atoms[0]][0].1;
//             let bond_right = model.bonds_with[indices_of_non_central_atoms[1]][0].1;
//
//             let central_atom_line_2 = central_atom.clone().name;
//
//             let lines_left = atom_lines(lone_left, &atom_left.name, Position::LEFT);
//             let lines_right = atom_lines(lone_right, &atom_right.name, Position::RIGHT);
//
//             //
//             //
//             //  ..  .. ..  ..
//             // :XX---XX ---XX:
//             //  ..   ..    ..
//             //
//             //
//
//             for i in 0..3 { // each line
//                 for j in 0..5 { // each block (left atom, bond, middle, bond, right)
//                     if j == 0 {
//                         print!("{}", lines_left[i]);
//                     } else if j == 1 {
//                         if i == 1 {
//                             match bond_left {
//                                 BondType::SINGLE => { print!("---") }
//                                 BondType::DOUBLE => { print!("===") }
//                                 BondType::TRIPLE => { print!("≡≡≡") }
//                             }
//                         } else {
//                             print!("  ");
//                         }
//
//                     } else if j == 2 {
//                         if i == 0 {
//                             print!(".. ..");
//                         } else if i == 1 {
//                             print!("{} ", central_atom_line_2);
//                         } else if i == 2 {
//                             print!(" ˙˙  ");
//                         } else {
//                             print!("  ");
//                         }
//                     } else if j == 3 {
//                         if i == 1 {
//                             match bond_right {
//                                 BondType::SINGLE => { print!("---") }
//                                 BondType::DOUBLE => { print!("===") }
//                                 BondType::TRIPLE => { print!("≡≡≡") }
//                             }
//                         } else {
//                             print!("  ");
//                         }
//
//                     } else if j == 4 {
//                         print!("{}", lines_right[i]);
//                     }
//                 }
//                 println!();
//             }
//         }
//         Geometry::Octahedral => {
//             let atom_left  = atoms_without_central.remove(0).borrow().clone();
//             let atom_right = atoms_without_central.remove(0).borrow().clone();
//             let atom_down_left = atoms_without_central.remove(0).borrow().clone();
//             let atom_down_right = atoms_without_central.remove(0).borrow().clone();
//             let atom_up_left = atoms_without_central.remove(0).borrow().clone();
//             let atom_up_right = atoms_without_central.remove(0).borrow().clone();
//
//             let lone_left = atom_left.lone;
//             let lone_right = atom_right.lone;
//             let lone_down_left = atom_down_left.lone;
//             let lone_down_right = atom_down_right.lone;
//             let lone_up_left = atom_up_left.lone;
//             let lone_up_right = atom_up_right.lone;
//
//             let bond_left = model.bonds_with[indices_of_non_central_atoms[0]][0].1;
//             let bond_right = model.bonds_with[indices_of_non_central_atoms[1]][0].1;
//             let bond_down_left = model.bonds_with[indices_of_non_central_atoms[2]][0].1;
//             let bond_down_right = model.bonds_with[indices_of_non_central_atoms[3]][0].1;
//             let bond_up_left = model.bonds_with[indices_of_non_central_atoms[4]][0].1;
//             let bond_up_right = model.bonds_with[indices_of_non_central_atoms[5]][0].1;
//
//             let central_atom_line = central_atom.name.clone();
//
//             let lines_left = atom_lines(lone_left, &atom_left.name, Position::LEFT);
//             let lines_right = atom_lines(lone_right, &atom_right.name, Position::RIGHT);
//             let lines_down_left = atom_lines(lone_down_left, &atom_down_left.name, Position::DOWN);
//             let lines_down_right = atom_lines(lone_down_right, &atom_down_right.name, Position::DOWN);
//             let lines_up_left = atom_lines(lone_up_left, &atom_up_left.name, Position::UP);
//             let lines_up_right = atom_lines(lone_up_right, &atom_up_right.name, Position::UP);
//
//             //   ..    ..
//             //  :XX:  :XX:
//             //    \   /
//             //  .. \ /  ..
//             // :XX--XX--XX:
//             //  .. / \  ..
//             //    /   \
//             //  :XX:  :XX:
//             //   ..    ..
//
//             for i in 0..7 {
//                 for j in 0..5 {
//                     if i == 0 {
//                         if j == 0 {
//                             print!(" {}", lines_up_left[0]);
//                         } else if j == 2 {
//                             print!("{}", lines_up_right[0]);
//                         } else {
//                             print!("  ");
//                         }
//                     } else if i == 1 {
//                         if j == 0 {
//                             print!(" {}", lines_up_left[1]);
//                         } else if j == 2 {
//                             print!("{}", lines_up_right[1]);
//                         } else {
//                             print!("  ");
//                         }
//                     } else if i == 2 {
//                         if j == 0 {
//                             match bond_up_left {
//                                 BondType::SINGLE => { print!("   \\ "); }
//                                 BondType::DOUBLE => { print!("   \\\\"); }
//                                 BondType::TRIPLE => {}
//                             }
//                             match bond_up_right {
//                                 BondType::SINGLE => { print!("  / "); }
//                                 BondType::DOUBLE => { print!("  //"); }
//                                 BondType::TRIPLE => {}
//                             }
//                         }
//                     } else if i == 3 {
//                         if j == 0 {
//                             print!("{}", lines_left[0]);
//                         } else if j == 1 {
//                             match bond_up_left {
//                                 BondType::SINGLE => { print!(" \\ "); }
//                                 BondType::DOUBLE => { print!(" \\\\"); }
//                                 BondType::TRIPLE => {}
//                             }
//                         } else if j == 2 {
//                             match bond_up_right {
//                                 BondType::SINGLE => { print!("/  "); }
//                                 BondType::DOUBLE => { print!("// "); }
//                                 BondType::TRIPLE => {}
//                             }
//                         } else if j == 3 {
//                             print!("{}", lines_right[0]);
//                         }
//                     } else if i == 4 {
//                         if j == 0 {
//                             print!("{}", lines_left[1]);
//                             match bond_left {
//                                 BondType::SINGLE => { print!("--"); }
//                                 BondType::DOUBLE => { print!("=="); }
//                                 BondType::TRIPLE => { print!("≡≡"); }
//                             }
//                             print!("{}",central_atom_line);
//                             match bond_right {
//                                 BondType::SINGLE => { print!("--"); }
//                                 BondType::DOUBLE => { print!("=="); }
//                                 BondType::TRIPLE => { print!("≡≡"); }
//                             }
//                             print!("{}", lines_right[1]);
//                         }
//                     } else if i == 5 {
//                         if j == 0 {
//                             print!("{}", lines_left[2]);
//                             match bond_down_left {
//                                 BondType::SINGLE => { print!(" / "); }
//                                 BondType::DOUBLE => { print!(" //"); }
//                                 BondType::TRIPLE => {}
//                             }
//                             match bond_down_right {
//                                 BondType::SINGLE => { print!("\\  "); }
//                                 BondType::DOUBLE => { print!("\\\\ "); }
//                                 BondType::TRIPLE => {}
//                             }
//                             print!("{}", lines_right[2]);
//                         }
//                     } else if i == 6 {
//                         if j == 0 {
//                             match bond_down_left {
//                                 BondType::SINGLE => { print!("   / "); }
//                                 BondType::DOUBLE => { print!("   //"); }
//                                 BondType::TRIPLE => {}
//                             }
//                             match bond_down_right {
//                                 BondType::SINGLE => { print!("  \\ "); }
//                                 BondType::DOUBLE => { print!("  \\\\"); }
//                                 BondType::TRIPLE => {}
//                             }
//                         }
//                     }
//                 }
//                 println!();
//             }
//
//             println!(" {}  {}", lines_down_left[0], lines_down_right[0]);
//             println!(" {}  {}", lines_down_left[1], lines_down_right[1]);
//         }
//
//         Geometry::SquarePyramidal => {
//             let atom_right = atoms_without_central.remove(0).borrow().clone();
//             let atom_down_left = atoms_without_central.remove(0).borrow().clone();
//             let atom_down_right = atoms_without_central.remove(0).borrow().clone();
//             let atom_up_left = atoms_without_central.remove(0).borrow().clone();
//             let atom_up_right = atoms_without_central.remove(0).borrow().clone();
//
//             let lone_right = atom_right.lone;
//             let lone_down_left = atom_down_left.lone;
//             let lone_down_right = atom_down_right.lone;
//             let lone_up_left = atom_up_left.lone;
//             let lone_up_right = atom_up_right.lone;
//
//             let bond_right = model.bonds_with[indices_of_non_central_atoms[0]][0].1;
//             let bond_down_left = model.bonds_with[indices_of_non_central_atoms[1]][0].1;
//             let bond_down_right = model.bonds_with[indices_of_non_central_atoms[2]][0].1;
//             let bond_up_left = model.bonds_with[indices_of_non_central_atoms[3]][0].1;
//             let bond_up_right = model.bonds_with[indices_of_non_central_atoms[4]][0].1;
//
//             let central_atom_line = central_atom.name.clone();
//
//             let lines_right = atom_lines(lone_right, &atom_right.name, Position::RIGHT);
//             let lines_down_left = atom_lines(lone_down_left, &atom_down_left.name, Position::DOWN);
//             let lines_down_right = atom_lines(lone_down_right, &atom_down_right.name, Position::DOWN);
//             let lines_up_left = atom_lines(lone_up_left, &atom_up_left.name, Position::UP);
//             let lines_up_right = atom_lines(lone_up_right, &atom_up_right.name, Position::UP);
//
//             //   ..    ..
//             //  :XX:  :XX:
//             //    \   /
//             //     \ /  ..
//             //     :XX--XX:
//             //     / \  ..
//             //    /   \
//             //  :XX:  :XX:
//             //   ..    ..
//
//             for i in 0..7 {
//                 for j in 0..5 {
//                     if i == 0 {
//                         if j == 0 {
//                             print!(" {}", lines_up_left[0]);
//                         } else if j == 2 {
//                             print!("{}", lines_up_right[0]);
//                         } else {
//                             print!("  ");
//                         }
//                     } else if i == 1 {
//                         if j == 0 {
//                             print!(" {}", lines_up_left[1]);
//                         } else if j == 2 {
//                             print!("{}", lines_up_right[1]);
//                         } else {
//                             print!("  ");
//                         }
//                     } else if i == 2 {
//                         if j == 0 {
//                             match bond_up_left {
//                                 BondType::SINGLE => { print!("   \\ "); }
//                                 BondType::DOUBLE => { print!("   \\\\"); }
//                                 BondType::TRIPLE => {}
//                             }
//                             match bond_up_right {
//                                 BondType::SINGLE => { print!("  / "); }
//                                 BondType::DOUBLE => { print!("  //"); }
//                                 BondType::TRIPLE => {}
//                             }
//                         } else {
//                             print!("   ");
//                         }
//                     } else if i == 3 {
//                         if j == 1 {
//                             match bond_up_left {
//                                 BondType::SINGLE => { print!(" \\ "); }
//                                 BondType::DOUBLE => { print!(" \\\\"); }
//                                 BondType::TRIPLE => {}
//                             }
//                         } else if j == 2 {
//                             match bond_up_right {
//                                 BondType::SINGLE => { print!("/  "); }
//                                 BondType::DOUBLE => { print!("// "); }
//                                 BondType::TRIPLE => {}
//                             }
//                         } else if j == 3 {
//                             print!("{}", lines_right[0]);
//                         } else {
//                             print!("   ");
//                         }
//                     } else if i == 4 {
//                         if j == 2 {
//                             print!(":");
//                             print!("{}", central_atom_line);
//                             match bond_right {
//                                 BondType::SINGLE => { print!("--"); }
//                                 BondType::DOUBLE => { print!("=="); }
//                                 BondType::TRIPLE => { print!("≡≡"); }
//                             }
//                             print!("{}", lines_right[1]);
//                         } else {
//                             print!("  ");
//                         }
//                     } else if i == 5 {
//                         if j == 1 {
//                             match bond_down_left {
//                                 BondType::SINGLE => { print!(" / "); }
//                                 BondType::DOUBLE => { print!(" //"); }
//                                 BondType::TRIPLE => {}
//                             }
//                             match bond_down_right {
//                                 BondType::SINGLE => { print!("\\  "); }
//                                 BondType::DOUBLE => { print!("\\\\ "); }
//                                 BondType::TRIPLE => {}
//                             }
//                             print!("{}", lines_right[2]);
//                         } else {
//                             print!("   ");
//                         }
//                     } else if i == 6 {
//                         if j == 0 {
//                             match bond_down_left {
//                                 BondType::SINGLE => { print!("   / "); }
//                                 BondType::DOUBLE => { print!("   //"); }
//                                 BondType::TRIPLE => {}
//                             }
//                             match bond_down_right {
//                                 BondType::SINGLE => { print!("  \\ "); }
//                                 BondType::DOUBLE => { print!("  \\\\"); }
//                                 BondType::TRIPLE => {}
//                             }
//                         }
//                     }
//                 }
//                 println!();
//             }
//
//             println!(" {}  {}", lines_down_left[0], lines_down_right[0]);
//             println!(" {}  {}", lines_down_left[1], lines_down_right[1]);
//         }
//
//         Geometry::SquarePlanar => {
//             //   ..    ..
//             //  :XX:  :XX:
//             //    \   /
//             //     \ /
//             //     :XX:
//             //     / \
//             //    /   \
//             //  :XX:  :XX:
//             //   ..    ..
//
//             let atom_down_left = atoms_without_central.remove(0).borrow().clone();
//             let atom_down_right = atoms_without_central.remove(0).borrow().clone();
//             let atom_up_left = atoms_without_central.remove(0).borrow().clone();
//             let atom_up_right = atoms_without_central.remove(0).borrow().clone();
//
//             let lone_down_left = atom_down_left.lone;
//             let lone_down_right = atom_down_right.lone;
//             let lone_up_left = atom_up_left.lone;
//             let lone_up_right = atom_up_right.lone;
//
//             let bond_down_left = model.bonds_with[indices_of_non_central_atoms[0]][0].1;
//             let bond_down_right = model.bonds_with[indices_of_non_central_atoms[1]][0].1;
//             let bond_up_left = model.bonds_with[indices_of_non_central_atoms[2]][0].1;
//             let bond_up_right = model.bonds_with[indices_of_non_central_atoms[3]][0].1;
//
//             let central_atom_line = central_atom.name.clone();
//
//             let lines_down_left = atom_lines(lone_down_left, &atom_down_left.name, Position::DOWN);
//             let lines_down_right = atom_lines(lone_down_right, &atom_down_right.name, Position::DOWN);
//             let lines_up_left = atom_lines(lone_up_left, &atom_up_left.name, Position::UP);
//             let lines_up_right = atom_lines(lone_up_right, &atom_up_right.name, Position::UP);
//
//             //   ..    ..
//             //  :XX:  :XX:
//             //    \   /
//             //     \ /  ..
//             //     :XX--XX:
//             //     / \  ..
//             //    /   \
//             //  :XX:  :XX:
//             //   ..    ..
//
//             for i in 0..7 {
//                 for j in 0..5 {
//                     if i == 0 {
//                         if j == 0 {
//                             print!(" {}", lines_up_left[0]);
//                         } else if j == 2 {
//                             print!("{}", lines_up_right[0]);
//                         } else {
//                             print!("  ");
//                         }
//                     } else if i == 1 {
//                         if j == 0 {
//                             print!(" {}", lines_up_left[1]);
//                         } else if j == 2 {
//                             print!("{}", lines_up_right[1]);
//                         } else {
//                             print!("  ");
//                         }
//                     } else if i == 2 {
//                         if j == 0 {
//                             match bond_up_left {
//                                 BondType::SINGLE => { print!("   \\ "); }
//                                 BondType::DOUBLE => { print!("   \\\\"); }
//                                 BondType::TRIPLE => {}
//                             }
//                             match bond_up_right {
//                                 BondType::SINGLE => { print!("  / "); }
//                                 BondType::DOUBLE => { print!("  //"); }
//                                 BondType::TRIPLE => {}
//                             }
//                         } else {
//                             print!("   ");
//                         }
//                     } else if i == 3 {
//                         if j == 1 {
//                             match bond_up_left {
//                                 BondType::SINGLE => { print!(" \\ "); }
//                                 BondType::DOUBLE => { print!(" \\\\"); }
//                                 BondType::TRIPLE => {}
//                             }
//                         } else if j == 2 {
//                             match bond_up_right {
//                                 BondType::SINGLE => { print!("/  "); }
//                                 BondType::DOUBLE => { print!("// "); }
//                                 BondType::TRIPLE => {}
//                             }
//                         } else {
//                             print!("   ");
//                         }
//                     } else if i == 4 {
//                         if j == 2 {
//                             print!(":");
//                             print!("{}", central_atom_line);
//                             print!(":");
//                         } else {
//                             print!("  ");
//                         }
//                     } else if i == 5 {
//                         if j == 1 {
//                             match bond_down_left {
//                                 BondType::SINGLE => { print!(" / "); }
//                                 BondType::DOUBLE => { print!(" //"); }
//                                 BondType::TRIPLE => {}
//                             }
//                             match bond_down_right {
//                                 BondType::SINGLE => { print!("\\  "); }
//                                 BondType::DOUBLE => { print!("\\\\ "); }
//                                 BondType::TRIPLE => {}
//                             }
//                         } else {
//                             print!("   ");
//                         }
//                     } else if i == 6 {
//                         if j == 0 {
//                             match bond_down_left {
//                                 BondType::SINGLE => { print!("   / "); }
//                                 BondType::DOUBLE => { print!("   //"); }
//                                 BondType::TRIPLE => {}
//                             }
//                             match bond_down_right {
//                                 BondType::SINGLE => { print!("  \\ "); }
//                                 BondType::DOUBLE => { print!("  \\\\"); }
//                                 BondType::TRIPLE => {}
//                             }
//                         }
//                     }
//                 }
//                 println!();
//             }
//
//             println!(" {}  {}", lines_down_left[0], lines_down_right[0]);
//             println!(" {}  {}", lines_down_left[1], lines_down_right[1]);
//         }
//
//         Geometry::PentagonalBipyramidal => {}
//         Geometry::PentagonalPyramidal => {}
//         Geometry::SquareAntiprismatic => {}
//         Geometry::PentagonalPlanar => {}
//     }
// }
//
// fn electron_geometry(model: &Model, min_ele_atom: &Atom) -> Result<Geometry, String> {
//     let num_of_atoms = model.atoms.len();
//
//     let num_of_lones = min_ele_atom.lone;
//
//     match (num_of_atoms, num_of_lones) {
//         (2, _) => Ok(Geometry::Linear2),
//         (3, 0) => Ok(Geometry::Linear),
//         (4, 0) => Ok(Geometry::TrigonalPlanar),
//         (3, 2) => Ok(Geometry::Bent1Lone),
//         (5, 0) => Ok(Geometry::Tetrahedral),
//         (4, 2) => Ok(Geometry::TrigonalPyramidal),
//         (3, 4) => Ok(Geometry::Bent2Lone),
//         (6, 0) => Ok(Geometry::TrigonalBipyramidal),
//         (5, 2) => Ok(Geometry::Seesaw),
//         (4, 4) => Ok(Geometry::TShape),
//         (3, 6) => Ok(Geometry::Linear3Lone),
//         (7, 0) => Ok(Geometry::Octahedral),
//         (6, 2) => Ok(Geometry::SquarePyramidal),
//         (5, 4) => Ok(Geometry::SquarePlanar),
//         (7, 2) => Ok(Geometry::PentagonalPyramidal),
//         (6, 4) => Ok(Geometry::PentagonalPlanar),
//         (8, 0) => Ok(Geometry::PentagonalBipyramidal),
//         (9, 0) => Ok(Geometry::SquareAntiprismatic),
//         _ => Err("No geometry found".parse().unwrap())
//     }
// }
//
// fn atom_lines(lones: i64, atom_name: &String, position : Position) -> Vec<String> {
//     let mut line_1 = String::new();
//     let mut line_2 = String::new();
//     let mut line_3 = String::new();
//
//     if lones == 0 {
//         match position {
//             Position::LEFT => {
//                 line_1.push_str("   ");
//                 line_2.push_str(" ");
//                 line_2.push_str(&atom_name.to_owned());
//                 line_3.push_str("   ");
//             }
//             Position::RIGHT => {
//                 line_1.push_str("  ");
//                 line_2.push_str(&atom_name.to_owned());
//                 line_3.push_str("  ");
//             }
//             Position::UP => {
//                 line_1.push_str("    ");
//                 line_2.push(' ');
//                 line_2.push_str(&atom_name.to_owned());
//                 line_2.push(' ');
//             }
//             Position::DOWN => {
//                 line_1.push(' ');
//                 line_1.push_str(&atom_name.to_owned());
//                 line_1.push(' ');
//                 line_2.push_str("    ");
//             }
//         }
//     } else if lones == 2 {
//         match position {
//             Position::LEFT => {
//                 line_1.push_str("   ");
//                 line_2.push_str(":");
//                 line_2.push_str(&atom_name.to_owned());
//                 line_3.push_str("   ");
//             }
//             Position::RIGHT => {
//                 line_1.push_str("  ");
//                 line_2.push_str(&atom_name.to_owned());
//                 line_2.push_str(":");
//                 line_3.push_str("  ");
//             }
//             Position::UP => {
//                 line_1.push_str(" .. ");
//                 line_2.push(' ');
//                 line_2.push_str(&atom_name.to_owned());
//                 line_2.push(' ');
//             }
//             Position::DOWN => {
//                 line_1.push(' ');
//                 line_1.push_str(&atom_name.to_owned());
//                 line_1.push(' ');
//                 line_2.push_str(" ˙˙ ");
//             }
//         }
//     } else if lones == 4 {
//         match position {
//             Position::LEFT => {
//                 line_1.push_str(" ..");
//                 line_2.push_str(" ");
//                 line_2.push_str(&atom_name.to_owned());
//                 line_3.push_str(" ˙˙");
//             }
//             Position::RIGHT => {
//                 line_1.push_str("..");
//                 line_2.push_str(&atom_name.to_owned());
//                 line_3.push_str("˙˙");
//             }
//             Position::UP => {
//                 line_1.push_str("    ");
//                 line_2.push(':');
//                 line_2.push_str(&atom_name.to_owned());
//                 line_2.push(':');
//             }
//             Position::DOWN => {
//                 line_1.push(':');
//                 line_1.push_str(&atom_name.to_owned());
//                 line_1.push(':');
//                 line_2.push_str("    ");
//             }
//         }
//
//     } else if lones == 6 {
//         match position {
//             Position::LEFT => {
//                 line_1.push_str(" ..");
//                 line_2.push_str(":");
//                 line_2.push_str(&atom_name.to_owned());
//                 line_3.push_str(" ˙˙");
//             }
//             Position::RIGHT => {
//                 line_1.push_str("..");
//                 line_2.push_str(&atom_name.to_owned());
//                 line_2.push_str(":");
//                 line_3.push_str("˙˙");
//             }
//             Position::UP => {
//                 line_1.push_str(" .. ");
//                 line_2.push(':');
//                 line_2.push_str(&atom_name.to_owned());
//                 line_2.push(':');
//             }
//             Position::DOWN => {
//                 line_1.push(':');
//                 line_1.push_str(&atom_name.to_owned());
//                 line_1.push(':');
//                 line_2.push_str(" ˙˙ ");
//             }
//         }
//     }
//
//     vec![line_1, line_2, line_3]
// }