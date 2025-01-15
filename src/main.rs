use std::cell::RefCell;
use std::env;
use std::collections::{HashMap, HashSet, VecDeque};
use std::iter::zip;
use std::rc::Rc;
use std::time::Instant;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct Element {
    name: String,
    valence: i64,
    electroneg: i64,
    id: i64
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct Atom {
    name: String,
    valence: i64,
    lone: i64,
    id: i64
}

impl Atom {
    fn new(name: &str, valence: i64, lone: i64, id: i64) -> AtomRef {
        Rc::new(RefCell::new(Atom {
            name: name.to_string(),
            valence,
            lone,
            id,
        }))
    }

    fn get_bonded_electrons_count(&self, bonds_with: &Vec<(AtomRef, BondType)>) -> i64 {
        bonds_with.iter().map(|(_, bond_type)| {
            match bond_type {
                BondType::SINGLE => 1,
                BondType::DOUBLE => 2,
                BondType::TRIPLE => 3
            }
        }).sum()
    }

    fn formal_charge(&self, bonds_with: &Vec<(AtomRef, BondType)>) -> i64 {
        self.valence - self.lone - self.get_bonded_electrons_count(&bonds_with)
    }

    fn electrons_to_octet(&self, bonds_with: &Vec<(AtomRef, BondType)>) -> i64 {
        8 - self.lone - self.get_bonded_electrons_count(&bonds_with) * 2
    }

    fn electrons_to_two(&self, bonds_with: &Vec<(AtomRef, BondType)>) -> i64 {
        2 - self.lone - self.get_bonded_electrons_count(&bonds_with) * 2
    }
}


#[derive(Clone, Copy, Debug)]
enum BondType {
    SINGLE,
    DOUBLE,
    TRIPLE
}

struct ParsedCompound {
    elements: HashSet<Element>, // set corresponding to each element
    charge: i64
}

type AtomRef = Rc<RefCell<Atom>>;

#[derive(Debug, Clone)]
struct Model {
    atoms: Vec<AtomRef>,
    bonds_with: Vec<Vec<(AtomRef, BondType)>>,
}

impl Model {
    fn new(atoms: Vec<AtomRef>) -> Self {
        Model {
            atoms: atoms.clone(),
            bonds_with: vec![Vec::new(); atoms.len()],
        }
    }

    fn add_bond(&mut self, atom_1: AtomRef, atom_2: AtomRef, bond: BondType) {
        let mut is_bond_possible = false;

        {
            let mut deref_atom_1 = atom_1.borrow_mut();
            let mut deref_atom_2 = atom_2.borrow_mut();

            match bond {
                BondType::SINGLE => {
                    if deref_atom_1.lone > 0 && deref_atom_2.lone > 0 {
                        is_bond_possible = true;
                        deref_atom_1.lone -= 1;
                        deref_atom_2.lone -= 1;
                    }
                }
                BondType::DOUBLE => {
                    if deref_atom_1.lone > 1 && deref_atom_2.lone > 1 {
                        is_bond_possible = true;
                        deref_atom_1.lone -= 2;
                        deref_atom_2.lone -= 2;
                    }
                }
                BondType::TRIPLE => {
                    if deref_atom_1.lone > 2 && deref_atom_2.lone > 2 {
                        is_bond_possible = true;
                        deref_atom_1.lone -= 3;
                        deref_atom_2.lone -= 3;
                    }
                }
            }
        }

        if is_bond_possible {
            let mut index_of_atom_1 = 0;
            let mut index_of_atom_2 = 0;

            for i in 0..self.atoms.len() {
                if self.atoms[i] == atom_1 {
                    index_of_atom_1 = i;
                }
                if self.atoms[i] == atom_2 {
                    index_of_atom_2 = i;
                }
            }

            let mut bonds_of_atom_1 = self.bonds_with.remove(index_of_atom_1);
            bonds_of_atom_1.push((atom_2, bond));
            self.bonds_with.insert(index_of_atom_1, bonds_of_atom_1);

            let mut bonds_of_atom_2 = self.bonds_with.remove(index_of_atom_2);
            bonds_of_atom_2.push((atom_1, bond));
            self.bonds_with.insert(index_of_atom_2, bonds_of_atom_2);
        }
    }

    fn remove_bond(&mut self, atom_1: AtomRef, atom_2: AtomRef) -> BondType {
        let mut index_of_atom_1 = 0;
        let mut index_of_atom_2 = 0;

        for i in 0..self.atoms.len() {
            if self.atoms[i] == atom_1 {
                index_of_atom_1 = i;
            }
            if self.atoms[i] == atom_2 {
                index_of_atom_2 = i;
            }
        }

        let mut bond = BondType::SINGLE;
        let mut bond_found = false;

        let mut bonds_of_atom_1 = self.bonds_with.remove(index_of_atom_1);
        for i in 0..bonds_of_atom_1.iter().len() {
            if bonds_of_atom_1[i].0 == atom_2 {
                bond_found = true;
                bond = bonds_of_atom_1.remove(i).1;
                break;
            }
        }
        self.bonds_with.insert(index_of_atom_1, bonds_of_atom_1);

        let mut bonds_of_atom_2 = self.bonds_with.remove(index_of_atom_2);
        for i in 0..bonds_of_atom_2.iter().len() {
            if bonds_of_atom_2[i].0 == atom_1 {
                bonds_of_atom_2.remove(i);
                break;
            }
        }
        self.bonds_with.insert(index_of_atom_2, bonds_of_atom_2);

        if bond_found {
            match bond {
                BondType::SINGLE => {
                    atom_1.borrow_mut().lone += 1;
                    atom_2.borrow_mut().lone += 1;
                }
                BondType::DOUBLE => {
                    atom_1.borrow_mut().lone += 2;
                    atom_2.borrow_mut().lone += 2;
                }
                BondType::TRIPLE => {
                    atom_1.borrow_mut().lone += 3;
                    atom_2.borrow_mut().lone += 3;
                }
            }
        }

        bond
    }
}


fn main() {

    let valences: HashMap<&str, i64> = HashMap::from([
        ("H", 1),  // Hydrogen
        ("He", 2),  // Helium
        ("Li", 1),  // Lithium
        ("Be", 2),  // Beryllium
        ("B", 3),  // Boron
        ("C", 4),  // Carbon
        ("N", 5),  // Nitrogen
        ("O", 6),  // Oxygen
        ("F", 7),  // Fluorine
        ("Ne", 8),  // Neon
        ("Na", 1),  // Sodium
        ("Mg", 2),  // Magnesium
        ("Al", 3),  // Aluminum
        ("Si", 4),  // Silicon
        ("P", 5),  // Phosphorus
        ("S", 6),  // Sulfur
        ("Cl", 7),  // Chlorine
        ("Ar", 8),  // Argon
        ("K", 1),  // Potassium
        ("Ca", 2),  // Calcium
        ("Ga", 3),  // Gallium
        ("Ge", 4),  // Germanium
        ("As", 5),  // Arsenic
        ("Se", 6),  // Selenium
        ("Br", 7),  // Bromine
        ("Kr", 8),  // Krypton
        ("Rb", 1),  // Rubidium
        ("Sr", 2),  // Strontium
        ("In", 3),  // Indium
        ("Sn", 4),  // Tin
        ("Sb", 5),  // Antimony
        ("Te", 6),  // Tellurium
        ("I", 7),  // Iodine
        ("Xe", 8),  // Xenon
        ("Cs", 1),  // Cesium
        ("Ba", 2),  // Barium
        ("Tl", 3),  // Thallium
        ("Pb", 4),  // Lead
        ("Bi", 5),  // Bismuth
        ("Po", 6),  // Polonium
        ("At", 7),  // Astatine
        ("Rn", 8),  // Radon
    ]);

    let electronegativities: HashMap<&str, i64> = HashMap::from([
        ("H", 220),
        ("He", 0), // Helium has no electronegativity.
        ("Li", 98),
        ("Be", 155),
        ("B", 204),
        ("C", 255),
        ("N", 304),
        ("O", 343),
        ("F", 398),
        ("Ne", 0), // Neon has no electronegativity.
        ("Na", 93),
        ("Mg", 143),
        ("Al", 161),
        ("Si", 190),
        ("P", 219),
        ("S", 258),
        ("Cl", 316),
        ("Ar", 0), // Argon has no electronegativity.
        ("K", 82),
        ("Ca", 100),
        ("Ga", 181),
        ("Ge", 200),
        ("As", 220),
        ("Se", 240),
        ("Br", 280),
        ("Kr", 0), // Krypton has negligible electronegativity.
        ("Rb", 82),
        ("Sr", 95),
        ("In", 160),
        ("Sn", 199),
        ("Sb", 220),
        ("Te", 221),
        ("I", 259),
        ("Xe", 0), // Xenon has negligible electronegativity.
        ("Cs", 79),
        ("Ba", 89),
        ("Tl", 190),
        ("Pb", 200),
        ("Bi", 207),
        ("Po", 220),
        ("At", 270),
        ("Rn", 0), // Radon has negligible electronegativity.
        ("Fr", 70),
        ("Ra", 90),
    ]);

    let args: Vec<String> = env::args().collect();
    let input_compound = parse_input(&args, &valences, &electronegativities);

    let now = Instant::now();
    let model_compound = build_model(&input_compound);
    dbg!(&model_compound);
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}

fn parse_input(_args : &[String], valences: &HashMap<&str, i64>, electronegativities: &HashMap<&str, i64>) -> ParsedCompound {
    let charge: i64 = _args[2].clone().parse().unwrap();

    let inputted_compound = _args[1].clone();

    let mut counts: Vec<i64> = vec![];

    let mut element_names: Vec<String> = vec![];
    let mut element_name = String::new();

    for (i, c) in inputted_compound.chars().enumerate() {
        if c.is_alphabetic() {
            if c.is_uppercase() {
                if i != 0 && inputted_compound.chars().nth(i-1).unwrap().is_alphabetic() {
                    counts.push(1);
                }
                if !element_name.is_empty() {
                    element_names.push(element_name.clone());
                    element_name.clear();
                }
            }
            element_name.push(c);
        } else if c.is_digit(10) {
            if !element_name.is_empty() {
                element_names.push(element_name.clone());
                element_name.clear();
            }
            counts.push(c.to_digit(10).unwrap() as i64);
        }
    }
    if !element_name.is_empty() {
        element_names.push(element_name);
        if inputted_compound.chars().nth(inputted_compound.chars().count() - 1).unwrap().is_alphabetic() {
            counts.push(1);
        }
    }

    let mut elements: HashSet<Element> = HashSet::new();

    let zipped = zip(element_names.iter(), counts.iter());
    zipped.for_each(|(element, count)|
        for i in 0..*count {
            let ele = Element {
                name: element.clone(),
                valence: *valences.get(element.as_str()).unwrap(),
                electroneg: *electronegativities.get(element.as_str()).unwrap(),
                id: i
            };
            elements.insert(ele);
        }
    );

    ParsedCompound { elements, charge }
}

fn build_model(input_compound: &ParsedCompound) -> Model {

    let charge = input_compound.charge;

    let min_electroneg = input_compound.elements.iter()
        .filter(|x| x.name != "H")
        .min_by(|a, b| a.electroneg.cmp(&b.electroneg)).unwrap();

    let mut min_atom: AtomRef = Atom::new("", 0, 0, 0);
    let mut atoms_vec: Vec<AtomRef> = Vec::new();

    // add all atoms to compound
    input_compound.elements.iter().for_each(|element| {
        let atom = Atom { name: element.name.clone(), valence: element.valence, lone: element.valence, id: element.id };
        let atomref = Rc::new(RefCell::new(atom));
        atoms_vec.push(atomref.clone());
        if min_electroneg.name == element.name && min_electroneg.id == element.id {
            min_atom = atomref.clone();
        }
    });


    let atoms_count = atoms_vec.len();

    let mut compound = Model::new(atoms_vec);


    // add bonds between atoms and atom with the least electronegativity
    for i in 0..atoms_count {
        if compound.atoms.get(i).unwrap().clone() != min_atom {
            compound.add_bond(compound.atoms.get(i).unwrap().clone(), min_atom.clone(), BondType::SINGLE);
        }
    }

    let mut queue: VecDeque<Model> = VecDeque::new();
    queue.push_front(compound.clone());
    while !queue.is_empty() {
        let mut curr = queue.pop_front().unwrap().clone();

        let mut electrons_to_full_map: HashMap<Atom, i64> = HashMap::new();
        let mut formal_charges: HashMap<Atom, i64> = HashMap::new();

        for i in 0..curr.atoms.len() {
            formal_charges.entry(curr.atoms[i].borrow().clone()).or_insert(curr.atoms[i].borrow().formal_charge(&curr.bonds_with[i].clone()));

            if curr.atoms[i] != min_atom {
                if curr.atoms[i].borrow().name == "H" || curr.atoms[i].borrow().name == "He" {
                    electrons_to_full_map.entry(curr.atoms[i].borrow().clone()).or_insert(curr.atoms[i].borrow().electrons_to_two(&curr.bonds_with[i].clone()));
                } else {
                    electrons_to_full_map.entry(curr.atoms[i].borrow().clone()).or_insert(curr.atoms[i].borrow().electrons_to_octet(&curr.bonds_with[i].clone()));
                }
            }
        }

        if electrons_to_full_map.values().sum::<i64>() == 0 && formal_charges.values().sum::<i64>() == charge {
            return curr;
        } else {
            for i in 0..atoms_count {
                let atom = curr.atoms.get(i).unwrap().clone();
                if atom != min_atom {
                    let bond = curr.remove_bond(atom, min_atom.clone());
                    match bond {
                        BondType::SINGLE => {
                            curr.add_bond(curr.atoms.get(i).unwrap().clone(), min_atom.clone(), BondType::DOUBLE);
                            queue.push_front(curr.clone());
                        }
                        BondType::DOUBLE => {
                            curr.add_bond(curr.atoms.get(i).unwrap().clone(), min_atom.clone(), BondType::TRIPLE);
                            queue.push_front(curr.clone());
                        }
                        BondType::TRIPLE => {}
                    }
                }
            }
        }
    }

    compound
}

fn test(input_compound: &ParsedCompound) -> Model {

    let min_electroneg = input_compound.elements.iter().filter(|x| x.name != "H").min_by(|a, b| a.electroneg.cmp(&b.electroneg)).unwrap();
    let mut min_atom: AtomRef = Atom::new("", 0, 0, 0);
    let mut atoms_vec: Vec<AtomRef> = Vec::new();

    // add all atoms to compound
    input_compound.elements.iter().for_each(|element| {
        let atom = Atom { name: element.name.clone(), valence: element.valence, lone: element.valence, id: element.id };
        let atomref = Rc::new(RefCell::new(atom.clone()));
        atoms_vec.push(atomref.clone());
        if min_electroneg.name == element.name && min_electroneg.id == element.id {
            min_atom = atomref.clone();
        }
    });

    dbg!(&atoms_vec.len());

    let mut compound = Model::new(atoms_vec);


    let atoms = compound.atoms.clone();

    for i in atoms.iter() {
        if *i == min_atom.clone() {
            continue;
        } else {
            compound.add_bond(i.clone(), min_atom.clone(), BondType::DOUBLE);
        }
    }

    compound
}

/*
first count the total number of valence electrons in the molecule,
then arrange the atoms with the least electronegative atom as the central atom,
and distribute the electrons around each atom to satisfy the octet rule
 by placing bonding pairs between atoms and lone pairs on individual atoms,
 while minimizing formal charges where possible
 */