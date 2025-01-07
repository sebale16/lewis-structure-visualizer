use std::env;
use std::collections::{HashMap, HashSet, VecDeque};
use std::iter::zip;

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
    bond_count: i64,
    id: i64
}

impl Atom {
    fn formal_charge(&self) -> i64{
        self.valence - self.lone - self.bond_count
    }
}

#[derive(Clone, Copy, Debug)]
enum BondType {
    SINGLE,
    DOUBLE,
    TRIPLE
}

struct ParsedCompound {
    elements: HashSet<Element>, // vector corresponding to each element
    charge: i64
}

struct ModelCompound {
    adjacency_list: HashMap<Atom, HashMap<Atom, BondType>>
}

impl ModelCompound {
    fn new() -> Self {
        ModelCompound {
            adjacency_list: HashMap::new()
        }
    }

    fn add_atom(&mut self, atom: &Atom) {
        self.adjacency_list.entry(atom.clone()).or_insert(HashMap::new());
    }

    fn add_bond(&mut self, atom1: &Atom, atom2: &Atom, bond: BondType) {
        self.adjacency_list
            .entry(atom1.clone())
            .or_insert_with(HashMap::new)
            .insert(atom2.clone(), bond);
        self.adjacency_list
            .entry(atom2.clone())
            .or_insert_with(HashMap::new)
            .insert(atom1.clone(), bond);
    }

    fn get_atoms(&self) -> HashSet<&Atom> {
        self.adjacency_list.keys().collect::<HashSet<&Atom>>()
    }

    fn contains_atom(&self, atom: &Atom) -> bool {
        self.adjacency_list.contains_key(atom)
    }

    fn neighbors(&self, atom: &Atom) -> Option<HashSet<&Atom>> {
        let set : HashSet<&Atom> = self.adjacency_list
            .get(atom)
            .unwrap()
            .keys()
            .collect();
        if set.is_empty() {
            None
        } else {
            Some(set)
        }
    }

    fn print_graph(&self) {
        for (atom, neighbors) in &self.adjacency_list {
            println!("{:?} -> {:?}", atom, neighbors);
        }
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

    let model_compound = build_model_compound(&input_compound).unwrap();
    model_compound.print_graph();
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

fn build_model_compound(input_compound: &ParsedCompound) -> Option<ModelCompound> {
    let mut compound = ModelCompound::new();

    // count total num of valence electrons
    let valence_electron_count: i64 = input_compound.elements
        .iter()
        .map(|element| element.valence)
        .sum();

    // determine least electronegative atom
    let min_electronegativity = input_compound.elements.iter().min_by(|a, b| a.electroneg.cmp(&b.electroneg)).unwrap();

    dbg!(valence_electron_count);
    dbg!(min_electronegativity);

    // add all atoms to compound
    input_compound.elements.iter().for_each(|element| {
        compound.add_atom(&Atom { name: element.name.clone(), valence: element.valence, lone: 0, bond_count: 0, id: element.id });
    });

    // determine bonds
    let formal_charge_total = 8;

    let mut queue: VecDeque<ModelCompound> = VecDeque::new();
    queue.push_front(compound);
    while !queue.is_empty() {
        let curr = queue.pop_front().unwrap();
        let mut curr_formal_charge_total = 0;
        curr.get_atoms().iter().for_each(|atom| {
            curr_formal_charge_total += atom.formal_charge();
        });
        if curr_formal_charge_total == formal_charge_total {
            return Some(curr);
        } else {

        }
    }

    None
}

/*
first count the total number of valence electrons in the molecule,
then arrange the atoms with the least electronegative atom as the central atom,
and distribute the electrons around each atom to satisfy the octet rule
 by placing bonding pairs between atoms and lone pairs on individual atoms,
 while minimizing formal charges where possible
 */