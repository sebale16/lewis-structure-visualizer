// mod display;

use std::cell::RefCell;
use std::env;
use std::collections::{HashMap, HashSet, VecDeque};
use std::iter::zip;
use std::rc::Rc;
// use crate::display::display;
use itertools::Itertools;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct Element {
    name: String,
    valence: u64,
    electroneg: u64,
    id: u64
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct Atom {
    name: String,
    valence: u64,
    lone: i64,
    id: u64
}

impl Atom {
    fn new(name: &str, valence: u64, lone: i64, id: u64) -> AtomRef {
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
        }).sum::<i64>()
    }

    fn formal_charge(&self, valence_list: &HashMap<&str, u64>, bonds_with: &Vec<(AtomRef, BondType)>) -> i64 {
        *valence_list.get(self.name.as_str()).unwrap() as i64 - self.lone - self.get_bonded_electrons_count(&bonds_with)
    }

    fn electrons_to_octet(&self, bonds_with: &Vec<(AtomRef, BondType)>) -> i64 {
        8 - self.lone - self.get_bonded_electrons_count(&bonds_with) * 2
    }

    fn electrons_to_six(&self, bonds_with: &Vec<(AtomRef, BondType)>) -> i64 {
        6 - self.lone - self.get_bonded_electrons_count(&bonds_with) * 2
    }

    fn electrons_to_four(&self, bonds_with: &Vec<(AtomRef, BondType)>) -> i64 {
        4 - self.lone - self.get_bonded_electrons_count(&bonds_with) * 2
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
    elements: Vec<Element>, // set corresponding to each element
    charge: i64
}

type AtomRef = Rc<RefCell<Atom>>;

#[derive(Debug, Clone)]
struct Model {
    atoms: Vec<AtomRef>,
    bonds_with: Vec<Vec<(AtomRef, BondType)>>,
    central_atom_id: usize
}

impl Model {
    fn new(atoms: Vec<AtomRef>, central_atom_id: usize) -> Self {
        Model {
            atoms: atoms.clone(),
            bonds_with: vec![Vec::new(); atoms.len()],
            central_atom_id
        }
    }

    fn add_bond(&mut self, atom_1: AtomRef, atom_2: AtomRef, bond: BondType) -> Result<bool, bool> {
        let mut is_bond_possible = false;

        {
            let deref_atom_1 = atom_1.borrow();
            let deref_atom_2 = atom_2.borrow();

            match bond {
                BondType::SINGLE => {
                    if deref_atom_1.lone > 0 && deref_atom_2.lone > 0 {
                        is_bond_possible = true;
                    }
                }
                BondType::DOUBLE => {
                    if deref_atom_1.lone > 1 && deref_atom_2.lone > 1 {
                        is_bond_possible = true;
                    }
                }
                BondType::TRIPLE => {
                    if deref_atom_1.lone > 2 && deref_atom_2.lone > 2 {
                        is_bond_possible = true;
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

        match is_bond_possible {
            true => Ok(true),
            false => Err(false)
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
        if bond_found {
            self.bonds_with.insert(index_of_atom_1, bonds_of_atom_1);
        }

        let mut bonds_of_atom_2 = self.bonds_with.remove(index_of_atom_2);
        for i in 0..bonds_of_atom_2.iter().len() {
            if bonds_of_atom_2[i].0 == atom_1 {
                bonds_of_atom_2.remove(i);
                break;
            }
        }
        if bond_found {
            self.bonds_with.insert(index_of_atom_2, bonds_of_atom_2);
        }

        bond
    }

    fn recalculate_lones(&mut self) {
        for i in 0..self.atoms.iter().len() {
            let mut atom = self.atoms[i].borrow_mut();
            atom.lone = atom.valence as i64 - atom.get_bonded_electrons_count(&self.bonds_with[i]);
        }
    }

    #[cfg(debug_assertions)]
    fn print_model(&self) {
        for i in 0..self.atoms.len() {
            println!("{:?} -> {:?}", self.atoms[i].borrow(), self.bonds_with[i].iter()
                .map(|(a, b)| {(a.borrow().clone(), *b)})
                .collect::<Vec<(Atom, BondType)>>());
        }
    }
}


fn main() {

    let valences: HashMap<&str, u64> = HashMap::from([
        ("H ", 1),  // Hydrogen
        ("He", 2),  // Helium
        ("Li", 1),  // Lithium
        ("Be", 2),  // Beryllium
        ("B ", 3),  // Boron
        ("C ", 4),  // Carbon
        ("N ", 5),  // Nitrogen
        ("O ", 6),  // Oxygen
        ("F ", 7),  // Fluorine
        ("Ne", 8),  // Neon
        ("Na", 1),  // Sodium
        ("Mg", 2),  // Magnesium
        ("Al", 3),  // Aluminum
        ("Si", 4),  // Silicon
        ("P ", 5),  // Phosphorus
        ("S ", 6),  // Sulfur
        ("Cl", 7),  // Chlorine
        ("Ar", 8),  // Argon
        ("K ", 1),  // Potassium
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
        ("I ", 7),  // Iodine
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

    let electronegativities: HashMap<&str, u64> = HashMap::from([
        ("H ", 220),
        ("He", 0), // Helium has no electronegativity.
        ("Li", 98),
        ("Be", 155),
        ("B ", 204),
        ("C ", 255),
        ("N ", 304),
        ("O ", 343),
        ("F ", 398),
        ("Ne", 0), // Neon has no electronegativity.
        ("Na", 93),
        ("Mg", 143),
        ("Al", 161),
        ("Si", 190),
        ("P ", 219),
        ("S ", 258),
        ("Cl", 316),
        ("Ar", 0), // Argon has no electronegativity.
        ("K ", 82),
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
        ("I ", 259),
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

    let can_expand_octet: HashMap<&str, bool> = HashMap::from([
        ("H ", false),
        ("He", false),
        ("Li", false),
        ("Be", false),
        ("B ", false),
        ("C ", false),
        ("N ", false),
        ("O ", false),
        ("F ", false),
        ("Ne", false),
        ("Na", false),
        ("Mg", false),
        ("Al", false),
        ("Si", true),
        ("P ", true),
        ("S ", true),
        ("Cl", true),
        ("Ar", false),
        ("K ", false),
        ("Ca", false),
        ("Ga", false),
        ("Ge", true),
        ("As", true),
        ("Se", true),
        ("Br", true),
        ("Kr", true),
        ("Rb", false),
        ("Sr", false),
        ("In", false),
        ("Sn", true),
        ("Sb", true),
        ("Te", true),
        ("I ", true),
        ("Xe", true),
        ("Cs", false),
        ("Ba", false),
        ("Tl", false),
        ("Pb", true),
        ("Bi", true),
        ("Po", true),
        ("At", true),
        ("Rn", true),
        ("Fr", false),
        ("Ra", false),
    ]);

    let args: Vec<String> = env::args().collect();
    let input_compound = parse_input(&args, &valences, &electronegativities);

    // todo change this
    // if (input_compound.elements.iter().map(|x| x.valence).sum::<u64>() as i64 - input_compound.charge) % 8 != 0 || input_compound.charge.abs() > 7 {
    //     panic!("Cannot create a stable compound!");
    // }
    //
    let model_compound = build_model(&input_compound, can_expand_octet, valences);
    // display(model_compound);
}

fn parse_input(_args : &[String], valences: &HashMap<&str, u64>, electronegativities: &HashMap<&str, u64>) -> ParsedCompound {
    let charge: i64 = _args[2].clone().parse().unwrap();

    let inputted_compound = _args[1].clone();

    let mut counts: Vec<u64> = vec![];

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
            counts.push(c.to_digit(10).unwrap() as u64);
        }
    }
    if !element_name.is_empty() {
        element_names.push(element_name);
        if inputted_compound.chars().nth(inputted_compound.chars().count() - 1).unwrap().is_alphabetic() {
            counts.push(1);
        }
    }

    let mut elements= vec![];

    let zipped = zip(element_names.iter(), counts.iter());
    zipped.for_each(|(element, count)|
        for i in 0..*count {
            let mut name = element.clone();
            if name.len() == 1 {
                name.push(' ');
            }
            let ele = Element {
                name: name.clone(),
                valence: *valences.get(name.as_str()).expect(&format!("{} element not \
                compatible, does not exist, or incorrect input format! Use a capital letter to denote \
                the start of an element's name and lower case letters for the second letter.", name)),
                electroneg: *electronegativities.get(name.as_str()).unwrap(),
                id: i
            };
            elements.push(ele);
        }
    );

    ParsedCompound { elements, charge }
}

fn distribute_charge(charge: i64, atoms_count: usize) -> Vec<Vec<i64>> {
    let abs_charge = charge.abs();

    let mut charge_distributions = vec![];

    let mut charge_0_dists = vec![];
    let mut charge_non_0_dists = vec![];

    // ex: charge = 0, atoms_count = 4, central_atom_index = 0
    // output = [-1, 1, 0, 0], [-1, 0, 1, 0], [-1, 0, 0, 1]
    //          [-2, 1, 1, 0], [-2, 1, 0, 1], [-2, 0, 1, 1], [-2, 2, 0, 0], [-2, 0, 2, 0], [-2, 0, 0, 2]

    if atoms_count == 2 {
        charge_0_dists.push(vec![-1, 1]);
        charge_0_dists.push(vec![1, -1]);
        charge_0_dists.push(vec![2, -2]);
        charge_0_dists.push(vec![-2, 2]);
    } else {
        for i in 1..=3 {
            let mut curr_distribution_minus_first = vec![0i64; atoms_count - 1];
            if i == 1 {
                curr_distribution_minus_first[0] = 1;
            } else if i == 2 {
                curr_distribution_minus_first[0] = 1;
                curr_distribution_minus_first[1] = 1;
            } else if i == 3 {
                curr_distribution_minus_first[0] = 2;
            }
            let curr_dist_minus_first_iter = curr_distribution_minus_first.iter();
            let perms = curr_dist_minus_first_iter.permutations(atoms_count - 1);
            let perms_as_set = perms.collect::<HashSet<_>>();
            for perm in perms_as_set {
                let mut curr_dist: Vec<i64> = perm.iter().map(|x| **x).collect();
                curr_dist.insert(0, -std::cmp::min(i, 2));
                charge_0_dists.push(curr_dist);
            }
        }
    }

    if charge != 0 {
        // ex: charge = -2, atoms_count = 4
        // output = [2, 0, 0, 0], [0, 2, 0, 0], [0, 0, 2, 0], [0, 0, 0, 2]
        //          [1, 1, 0, 0], [1, 0, 1, 0], [1, 0, 0, 1], [0, 1, 1, 0], [0, 1, 0, 1], [0, 0, 1, 1]
        // 10 vecs

        // ex: charge = -3, atoms_count = 4
        // output = [3, 0, 0, 0], [0, 3, 0, 0], [0, 0, 3, 0], [0, 0, 0, 3]
        //          [2, 1, 0, 0], [2, 0, 1, 0], [2, 0, 0, 1], [1, 2, 0, 0], [0, 2, 1, 0], [0, 2, 0, 1], [1, 0, 2, 0], [0, 1, 2, 0], [0, 0, 2, 1], [1, 0, 0, 2], [0, 1, 0, 2], [0, 0, 1, 2],
        //          [1, 1, 1, 0], [1, 1, 0, 1], [1, 0, 1, 1], [0, 1, 1, 1]
        // 20 vecs

        for i in 0..abs_charge {
            let mut v = vec![0i64; atoms_count];
            let mut count = abs_charge - i;
            let mut sum = 0;
            let mut index = 0;

            while sum < abs_charge {
                v[index] = count;
                sum += count;
                index += 1;
                if count != 1 {
                    count -= 1;
                }
            }

            let perms_set = v.iter().permutations(atoms_count).collect::<HashSet<Vec<_>>>();

            for perm in perms_set {
                if charge > 0 {
                    let curr_dist: Vec<i64> = perm.iter().map(|x| -**x).collect();
                    charge_non_0_dists.push(curr_dist);
                } else {
                    let curr_dist: Vec<i64> = perm.iter().map(|x| **x).collect();
                    charge_non_0_dists.push(curr_dist);
                }
            }
        }
    }

    if charge_non_0_dists.len() > 0 {
        charge_distributions = charge_0_dists.iter()
            .flat_map(|a| charge_non_0_dists.iter().map(move |b| a.iter().zip(b.iter()).map(|(x, y)| x+y).collect()))
            .collect();
        for i in charge_non_0_dists.iter() {
            charge_distributions.insert(0, i.clone());
        }
    } else {
        charge_distributions = charge_0_dists;
        charge_distributions.insert(0, vec![0i64; atoms_count]);
    }



    charge_distributions
}

fn determine_formal_charge_distribution(models: Vec<Model>, valences: HashMap<&str, u64>) -> Model {
    let mut min_abs_formal_charge_index = 0;
    let mut min_abs_formal_charge = 8;

    for i in 0..models.len() {
        let mut curr_model = models[i].clone();
        curr_model.recalculate_lones();
        let mut curr_model_formal = 0;
        for j in 0..curr_model.atoms.len() {
            curr_model_formal += curr_model.atoms[j].borrow().formal_charge(&valences, &curr_model.bonds_with[j]).abs();
        }
        if curr_model_formal < min_abs_formal_charge {
            min_abs_formal_charge = curr_model_formal;
            min_abs_formal_charge_index = i;
        }
    }

    models.get(min_abs_formal_charge_index).expect("No model found!").clone()
}

fn build_model(input_compound: &ParsedCompound, can_expand: HashMap<&str, bool>, valence_map: HashMap<&str, u64>) -> Model {
    let charge = input_compound.charge;

    let min_electroneg = input_compound.elements.iter()
        .filter(|x| x.name != "H ")
        .min_by(|a, b| a.electroneg.cmp(&b.electroneg))
        .unwrap_or(input_compound.elements.iter().next().unwrap());

    let mut central_atoms = vec![];

    let mut compounds_vec = vec![];

    let atoms_count = input_compound.elements.len();

    let charge_distributions = distribute_charge(charge, atoms_count);

    for i in 0..charge_distributions.len() {
        let mut atoms_vec: Vec<AtomRef> = vec![];

        let mut charge_dist_cloned = charge_distributions[i].clone();

        for i in 0..input_compound.elements.iter().len() {
            if input_compound.elements[i].name == "H " || input_compound.elements[i].name == "He" || input_compound.elements[i].name == "Be" {
                if charge_dist_cloned[i] != 0 {
                    charge_dist_cloned = vec![0; atoms_count];
                }
            }
        }

        for j in 0..input_compound.elements.iter().len() {
            let element = input_compound.elements[j].clone();

            let atom = Atom {
                name: element.name.clone(),
                valence: (element.valence as i64 + charge_dist_cloned[j]) as u64,
                lone: element.valence as i64 + charge_dist_cloned[j],
                id: element.id
            };

            let atomref = Rc::new(RefCell::new(atom));
            if min_electroneg.name == element.name && min_electroneg.id == element.id {
                central_atoms.push(atomref.clone());
                atoms_vec.insert(0, atomref.clone());
            } else {
                atoms_vec.push(atomref.clone());
            }
        }

        compounds_vec.push(Model::new(atoms_vec.clone(), i));
    }

    let mut queue: VecDeque<Model> = VecDeque::new();

    for i in 0..compounds_vec.len() {
        let central_atom = central_atoms[i].clone();
        // add bonds between atoms and atom with the least electronegativity
        for j in 0..atoms_count {
            let atomref = compounds_vec[i].atoms.get(j).unwrap().clone();
            if !central_atoms.contains(&atomref) {
                compounds_vec[i].add_bond(atomref, central_atom.clone(), BondType::SINGLE).unwrap();
            }
        }

        compounds_vec[i].recalculate_lones();
        if compounds_vec[i].atoms.iter().all(|x| x.borrow().lone >= 0) {
            queue.push_front(compounds_vec[i].clone());
        }
    }

    let mut fitting_models: Vec<Model> = vec![];

    while queue.len() > 0 {

        let mut curr = queue.pop_front().unwrap();
        let new_central_atom = central_atoms[curr.central_atom_id].clone();

        // after bonding
        curr.recalculate_lones();

        let mut electrons_to_full_map: HashMap<Atom, i64> = HashMap::new();

        for i in 0..curr.atoms.len() {
            if curr.atoms[i].borrow().name == "H " || curr.atoms[i].borrow().name == "He" {
                electrons_to_full_map.entry(curr.atoms[i].borrow().clone()).or_insert(curr.atoms[i].borrow().electrons_to_two(&curr.bonds_with[i].clone()));
            } else if curr.atoms[i].borrow().name == "Be" && atoms_count == 3 {
                electrons_to_full_map.entry(curr.atoms[i].borrow().clone()).or_insert(curr.atoms[i].borrow().electrons_to_four(&curr.bonds_with[i].clone()));
            } else if curr.atoms[i].borrow().name == "Al" || curr.atoms[i].borrow().name == "B " || curr.atoms[i].borrow().name == "C " && atoms_count == 4 {
                electrons_to_full_map.entry(curr.atoms[i].borrow().clone()).or_insert(curr.atoms[i].borrow().electrons_to_six(&curr.bonds_with[i].clone()));
            } else if !(can_expand.get(&curr.atoms[i].borrow().name.as_str()).unwrap().clone() && curr.atoms[i].borrow().eq(&new_central_atom.borrow())) {
                electrons_to_full_map.entry(curr.atoms[i].borrow().clone()).or_insert(curr.atoms[i].borrow().electrons_to_octet(&curr.bonds_with[i].clone()));
            }
        }

        if electrons_to_full_map.values().all(|&x| x == 0)  {
            fitting_models.push(curr);
        } else {
            for i in 0..atoms_count {
                let atom = curr.atoms.get(i).unwrap().clone();
                if atom != new_central_atom {
                    let mut cloned_curr = curr.clone(); // generate new model for each bond modification

                    let bond = cloned_curr.remove_bond(atom.clone(), new_central_atom.clone());

                    // recalculate lone electron amt before bonding
                    cloned_curr.recalculate_lones();

                    match bond {
                        BondType::SINGLE => {
                            match cloned_curr.add_bond(atom.clone(), new_central_atom.clone(), BondType::DOUBLE) {
                                Ok(_) => { queue.push_back(cloned_curr.clone()); }
                                Err(_) => { cloned_curr.add_bond(atom.clone(), new_central_atom.clone(), BondType::SINGLE).unwrap(); }
                            }
                        }
                        BondType::DOUBLE => {
                            match cloned_curr.add_bond(atom.clone(), new_central_atom.clone(), BondType::TRIPLE) {
                                Ok(_) => { queue.push_back(cloned_curr.clone()); }
                                Err(_) => { cloned_curr.add_bond(atom.clone(), new_central_atom.clone(), BondType::DOUBLE).unwrap(); }
                            }
                        }
                        BondType::TRIPLE => {}
                    }

                }
            }
        }
    }
    determine_formal_charge_distribution(fitting_models, valence_map)
}