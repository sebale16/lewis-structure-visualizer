use std::cell::RefCell;
use std::env;
use std::fs::File;
use std::iter::zip;
use std::rc::Rc;
use csv::{Reader};
use serde::{Serialize, Deserialize};
use std::io::BufWriter;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Element {
    name: String,
    electroneg: u32,
    config: Vec<String>,
    id: u32
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Atom {
    name: String,
    valence: u32,
    lone: u32,
    hybridization: Hybridization,
    spd_orbitals: Vec<u8>,
    p_orbitals: Vec<u8>,
    id: u32
}

#[derive(Clone, Copy, Debug)]
pub enum BondType {
    SIGMA,
    PI,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Hybridization {
    S,
    SP,
    SP2,
    SP3,
    SP3D,
    SP3D2,
    SP3D3,
    SP3D4,
    SP3D5,
}

impl Hybridization {
    fn next(self) -> Option<Hybridization> {
        match self {
            Hybridization::S => Some(Hybridization::SP),
            Hybridization::SP => Some(Hybridization::SP2),
            Hybridization::SP2 => Some(Hybridization::SP3),
            Hybridization::SP3 => Some(Hybridization::SP3D),
            Hybridization::SP3D => Some(Hybridization::SP3D2),
            Hybridization::SP3D2 => Some(Hybridization::SP3D3),
            Hybridization::SP3D3 => Some(Hybridization::SP3D4),
            Hybridization::SP3D4 => Some(Hybridization::SP3D5),
            Hybridization::SP3D5 => None
        }
    }
    fn before(self) -> Option<Hybridization> {
        match self {
            Hybridization::S => None,
            Hybridization::SP => Some(Hybridization::S),
            Hybridization::SP2 => Some(Hybridization::SP),
            Hybridization::SP3 => Some(Hybridization::SP2),
            Hybridization::SP3D => Some(Hybridization::SP3),
            Hybridization::SP3D2 => Some(Hybridization::SP3D),
            Hybridization::SP3D3 => Some(Hybridization::SP3D2),
            Hybridization::SP3D4 => Some(Hybridization::SP3D3),
            Hybridization::SP3D5 => Some(Hybridization::SP3D4),
        }
    }
}

#[derive(Debug)]
pub struct ParsedMolecule {
    name: String,
    elements: Vec<Element>,
    charge: i32
}

pub type AtomRef = Rc<RefCell<Atom>>;

#[derive(Debug, Clone)]
pub struct Model {
    name: String,
    atoms: Vec<AtomRef>,
    bonds_with: Vec<Vec<(AtomRef, BondType)>>,
}

#[derive(Serialize, Deserialize)]
struct EntryBond {
    name: String,
    id: u32,
    bond_type: String,
}

#[derive(Serialize, Deserialize)]
struct EntryAtom {
    name: String,
    valence: u32,
    lone: u32,
    id: u32,
    hybridization: String,
    bonds_with: Vec<EntryBond>,
    p_orbitals: Vec<u8>,
    spd_orbitals: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct EntryModel {
    name: String,
    atoms: Vec<EntryAtom>,
}

impl Model {
    fn new(name: String, atoms: Vec<AtomRef>) -> Self {
        Model {
            name: name,
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
                BondType::SIGMA => {
                    if deref_atom_1.spd_orbitals.contains(&(1u8)) && deref_atom_2.spd_orbitals.contains(&(1u8)) {
                        is_bond_possible = true;
                        if let Some(index) = deref_atom_1.spd_orbitals.iter().position(|&x| x == 1) {
                            deref_atom_1.spd_orbitals[index] = 2;
                            deref_atom_1.lone -= 1;
                        }
                        if let Some(index) = deref_atom_2.spd_orbitals.iter().position(|&x| x == 1) {
                            deref_atom_2.spd_orbitals[index] = 2;
                            deref_atom_2.lone -= 1;
                        }
                    }
                }
                BondType::PI => {
                    if deref_atom_1.p_orbitals.contains(&(1u8)) && deref_atom_2.p_orbitals.contains(&(1u8)) {
                        is_bond_possible = true;
                        if let Some(index) = deref_atom_1.p_orbitals.iter().position(|&x| x == 1) {
                            deref_atom_1.p_orbitals[index] = 2;
                            deref_atom_1.lone -= 1;
                        }
                        if let Some(index) = deref_atom_2.p_orbitals.iter().position(|&x| x == 1) {
                            deref_atom_2.p_orbitals[index] = 2;
                            deref_atom_2.lone -= 1;
                        }
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

    pub fn print_model(&self) {
        for i in 0..self.atoms.len() {
            println!("{:?} -> {:?}\n", self.atoms[i].borrow(), self.bonds_with[i].iter()
                .map(|(a, b)| {(a.borrow().clone(), *b)})
                .collect::<Vec<(Atom, BondType)>>());
        }
    }

    // write Model to json file
    pub fn write_to_json(&self, path: String) -> std::io::Result<()> { 
        // convert current struct to Entry struct so that it can be serialized
        // each Entry represents one atom in the model
        let entries = self.atoms.iter().enumerate().map(|(i, a)| {
            let atom = a.borrow();
            EntryAtom {
                name: atom.name.clone(),
                valence: atom.valence,
                lone: atom.lone,
                id: atom.id,
                hybridization: format!("{:?}", atom.hybridization),
                bonds_with: self.bonds_with[i].iter()
                    .map(|(a, b)| { EntryBond {name: a.borrow().name.clone(), id: a.borrow().id, bond_type: format!("{:?}", b)}})
                    .collect::<Vec<_>>(),
                p_orbitals: atom.p_orbitals.clone(),
                spd_orbitals: atom.spd_orbitals.clone(),
            }
        }).collect::<Vec<_>>();
        let entry_model: EntryModel = EntryModel {
            name: self.name.clone(),
            atoms: entries,
        };

        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &entry_model)?;
        Ok(())
    }
}

pub fn read_element_csv(file_path: &str, element_names: Vec<String>) -> Vec<Element> {
    let mut elements: Vec<Element> = Vec::new();

    for element_name in element_names {
        let file = File::open(file_path).unwrap();
        let mut rdr = Reader::from_reader(file);
        for result in rdr.records() {
            let record = result.unwrap();

            let mut symbol = record[1].replace("\t", "").replace(" ", "");

            if element_name == symbol {
                if symbol.len() == 1 {
                    symbol.push(' ');
                }

                let mut electron_config = record[5].split(' ').collect::<Vec<&str>>();
                electron_config.remove(electron_config.len() - 1);

                let mut changed_electron_config: Vec<String> = electron_config.iter().map(|s| s.to_string()).collect();

                if changed_electron_config.len() > 1 {
                    changed_electron_config.remove(0);
                    if changed_electron_config[0].contains("d") && changed_electron_config[1].contains("p") {
                        changed_electron_config.remove(0);
                    }
                }

                let electronegativity = (&record[6].parse::<f64>().unwrap_or(0.0) * 100.0) as u32;

                let mut element_with_same_name_index = -1;
                let mut element_with_same_name_id = -1;

                for i in 0..elements.iter().len() {
                    if elements.get(i).unwrap().name == *symbol {
                        element_with_same_name_index = i as i8;
                        element_with_same_name_id = elements.get(i).unwrap().id as i8;
                    }
                }

                if element_with_same_name_index != -1 {
                    elements.push(Element {name: symbol.clone(), electroneg: electronegativity, config: changed_electron_config.clone(), id: (element_with_same_name_id + 1) as u32 })
                } else {
                    elements.push(Element {name: symbol.clone(), electroneg: electronegativity, config: changed_electron_config.clone(), id: 0})
                }
            }
        }
    }

    elements
}

pub fn parse_input(args : &[String]) -> ParsedMolecule {
    let charge: i32 = args[2].clone().parse().unwrap();

    let inputted_molecule = args[1].clone();

    let mut counts: Vec<u32> = vec![];

    let mut element_names: Vec<String> = vec![];
    let mut element_name = String::new();

    for (i, c) in inputted_molecule.chars().enumerate() {
        if c.is_alphabetic() {
            if c.is_uppercase() {
                if i != 0 && inputted_molecule.chars().nth(i - 1).unwrap().is_alphabetic() {
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
            counts.push(c.to_digit(10).unwrap() as u32);
        }
    }
    if !element_name.is_empty() {
        element_names.push(element_name);
        if inputted_molecule.chars().nth(inputted_molecule.chars().count() - 1).unwrap().is_alphabetic() {
            counts.push(1);
        }
    }

    let mut element_names_counted = vec![];

    let zipped = zip(element_names.iter(), counts.iter());
    zipped.for_each(|(element, count)|
        for _ in 0..*count {
            element_names_counted.push(element.clone());
        }
    );

    let elements = read_element_csv(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../data/data.csv").to_str().unwrap(),
        element_names_counted
    );

    ParsedMolecule { name: format!("{}_{}", args[1].clone(), args[2]), elements, charge }
}

pub fn hybridize(valence: u8, which: Hybridization, central_atoms_bonds: Vec<(AtomRef, BondType)>) -> Vec<u8> {
    let mut hybridized = vec![];
    let mut bonds_count = central_atoms_bonds.len();
    match which {
        Hybridization::S => hybridized.push(0),
        Hybridization::SP => hybridized = vec![0; 2],
        Hybridization::SP2 => hybridized = vec![0; 3],
        Hybridization::SP3 => hybridized = vec![0; 4],
        Hybridization::SP3D => hybridized = vec![0; 5],
        Hybridization::SP3D2 => hybridized = vec![0; 6],
        Hybridization::SP3D3 => hybridized = vec![0; 7],
        Hybridization::SP3D4 => hybridized = vec![0; 8],
        Hybridization::SP3D5 => hybridized = vec![0; 9],
    }

    let length = hybridized.len();
    let mut added_valence = valence;
    for i in 0..valence {
        if added_valence > 0 {
            hybridized[i as usize % length] += 1;
            added_valence -= 1;
        } else {
            break;
        }
    }

    // add electrons from bonds
    for i in 0..length {
        if hybridized[i] == 1 && bonds_count > 0 {
            hybridized[i] = 2;
            bonds_count -= 1;
        }
    }
    hybridized
}

pub fn build_model(input_molecule: &ParsedMolecule) -> Model {
    let mut charge = input_molecule.charge.abs();

    let min_electroneg = input_molecule.elements.iter()
        .filter(|x| x.name != "H ")
        .min_by(|a, b| a.electroneg.cmp(&b.electroneg))
        .unwrap_or(input_molecule.elements.iter().next().unwrap());

    let bond_all_to_central = |atoms_count: usize, molecule: &mut Model, central_atom: AtomRef, bond_type: BondType| {
        for i in 0..atoms_count {
            let atomref = molecule.atoms.get(i).unwrap().clone();
            let mut has_central = false;
            for j in 0..molecule.bonds_with[i].len() {
                if molecule.bonds_with[i][j].0 == central_atom {
                    has_central = true;
                }
            }
            match bond_type {
                BondType::SIGMA => {
                    if central_atom.clone() != atomref && !has_central {
                        molecule.add_bond(atomref, central_atom.clone(), bond_type);
                    }
                }
                BondType::PI => {
                    if central_atom.clone() != atomref {
                        molecule.add_bond(atomref, central_atom.clone(), bond_type);
                    }
                }
            }

        }
    };

    let mut central_atom : AtomRef = Rc::new(RefCell::new(Atom {
        name: "".to_string(),
        valence: 0,
        lone: 0,
        hybridization: Hybridization::S,
        spd_orbitals: vec![],
        p_orbitals: vec![],
        id: 0,
    }));

    let atoms_count = input_molecule.elements.len();

    let mut atoms_vec: Vec<AtomRef> = vec![];
    let mut central_atom_index = 0;
    for j in 0..input_molecule.elements.iter().len() {

        let element = input_molecule.elements[j].clone();
        let mut valence_count = 0;

        if element.config.len() < 3 {
            let s_orbital_count = element.config[0].chars().last().unwrap().to_digit(10).unwrap();
            let p_orbital_count = element.config.get(1).unwrap_or(&"0".to_string()).chars().last().unwrap().to_digit(10).unwrap();
            valence_count += s_orbital_count + p_orbital_count;
        } else {
            let s_orbital_count = element.config[1].chars().last().unwrap().to_digit(10).unwrap();
            let p_orbital_count = element.config[2].chars().last().unwrap().to_digit(10).unwrap();
            valence_count += s_orbital_count + p_orbital_count;
        }

        let atom = Atom {
            name: element.name.clone(),
            valence: valence_count as u32,
            lone: valence_count as u32,
            hybridization: match valence_count {
                0 | 1 => Hybridization::S,
                2 => Hybridization::SP,
                3 => Hybridization::SP2,
                _ => Hybridization::SP3,
            },
            spd_orbitals: match valence_count {
                0 | 1 => hybridize(valence_count as u8, Hybridization::S, vec![]),
                2 => hybridize(valence_count as u8, Hybridization::SP, vec![]),
                3 => hybridize(valence_count as u8, Hybridization::SP2, vec![]),
                _ => hybridize(valence_count as u8, Hybridization::SP3, vec![]),
            },
            p_orbitals: vec![],
            id: element.id
        };

        let atomref = Rc::new(RefCell::new(atom));
        if min_electroneg.name == element.name && min_electroneg.id == element.id {
            central_atom = atomref.clone();
            central_atom_index = j;
        }
        atoms_vec.push(atomref.clone());
    }

    let mut molecule = Model::new(input_molecule.name.clone(), atoms_vec.clone());

    bond_all_to_central(atoms_count, &mut molecule, central_atom.clone(), BondType::SIGMA);

    // if all atoms are bonded but are missing electrons, create double/triple bonds
    // else hybridize further to allow for more bonding slots

    // if not all atoms are bonded, hybridize central atom further, try bonding again. keep going until all atoms are bonded to central atom
    while !molecule.bonds_with.iter().all(|x| x.len() > 0) {
        let next_hyb = central_atom.borrow().clone().hybridization.next().unwrap();
        let new_spds = hybridize(central_atom.borrow().clone().valence as u8, next_hyb.clone(), molecule.bonds_with[central_atom_index].clone()).clone();
        central_atom.borrow_mut().spd_orbitals = new_spds;
        central_atom.borrow_mut().hybridization = next_hyb;

        bond_all_to_central(atoms_count, &mut molecule, central_atom.clone(), BondType::SIGMA);
    }

    // once all atoms are bonded to central atom, take out hybridized orbitals with only one electron and make them p orbitals
    for i in 0..atoms_count {
        let mut spds = molecule.atoms[i].borrow().clone().spd_orbitals;
        let mut p_orbs = vec![];
        for j in 0..spds.len() {
            if spds[j] == 1 {
                spds[j] = 0;
                p_orbs.push(1);
            }
        }
        let mut atom = molecule.atoms[i].borrow_mut();
        atom.spd_orbitals = spds.iter().filter(|&&x| x != 0).copied().collect();
        atom.p_orbitals = p_orbs.clone();
        let mut new_hy = atom.hybridization.clone();
        for _ in 0..p_orbs.len() {
            new_hy = new_hy.before().unwrap();
        }
        atom.hybridization = new_hy;
    }

    // bond each extra p orbital to a central p orbital
    // bonded p orbitals (pi bonds) shown by a "ghost" electron in p_orbitals array to make it 2
    while central_atom.borrow().p_orbitals.contains(&1u8) {
        // search for atom that has an empty slot in p_orbitals, meaning it only has 1 electron
        bond_all_to_central(atoms_count, &mut molecule, central_atom.clone(), BondType::PI);
    }

    // if excess p_orbitals, check charge
    // if charge is 0, "give" p orbital to central atom
    // if charge is not 0, "give" p orbital back to corresponding atom and add missing number of electrons

    if charge == 0 {
        while molecule.atoms.iter().filter(|x| x.borrow().p_orbitals.contains(&1)).collect::<Vec<_>>().len() > 0 {
            if let Some(index) = molecule.atoms.iter().position(|x| x.borrow().p_orbitals.contains(&1u8)) {
                let mut modified_outer_atom = molecule.atoms[index].borrow_mut();
                let mut modified_central_atom = central_atom.borrow_mut();
                let new_hy_central = modified_central_atom.clone().hybridization.before().unwrap();
                let new_hy_outer = modified_outer_atom.clone().hybridization.next().unwrap();

                modified_outer_atom.p_orbitals.remove(0);
                modified_outer_atom.spd_orbitals.push(2);
                modified_outer_atom.hybridization = new_hy_outer;
                modified_outer_atom.lone += 1;
                modified_central_atom.spd_orbitals.pop();
                modified_central_atom.p_orbitals.push(1);
                modified_central_atom.hybridization = new_hy_central;
                modified_central_atom.lone -= 1;
            }
            bond_all_to_central(atoms_count, &mut molecule, central_atom.clone(), BondType::PI);
        }
    } else {
        while molecule.atoms.iter().filter(|x| x.borrow().p_orbitals.contains(&1u8)).collect::<Vec<_>>().len() > 0 {
            if charge > 0 {
                // change unbonded ps to lone pairs
                if let Some(index) = molecule.atoms.iter().position(|x| x.borrow().p_orbitals.contains(&1u8)) {
                    let mut modified_outer_atom = molecule.atoms[index].borrow_mut();
                    let new_hy = modified_outer_atom.clone().hybridization.next().unwrap();
                    modified_outer_atom.p_orbitals.remove(0);
                    modified_outer_atom.spd_orbitals.push(2);
                    modified_outer_atom.hybridization = new_hy;
                    modified_outer_atom.lone += 1;
                    charge -= 1;
                }
            } else {
                if let Some(index) = molecule.atoms.iter().position(|x| x.borrow().p_orbitals.contains(&1u8)) {
                    let mut modified_outer_atom = molecule.atoms[index].borrow_mut();
                    let mut modified_central_atom = central_atom.borrow_mut();
                    let new_hy_central = modified_central_atom.clone().hybridization.before().unwrap();
                    modified_central_atom.spd_orbitals.pop();
                    modified_central_atom.p_orbitals.push(1);
                    modified_central_atom.p_orbitals.push(1);
                    modified_central_atom.hybridization = new_hy_central;
                    modified_outer_atom.p_orbitals.remove(0);
                    modified_outer_atom.p_orbitals.push(1);
                }
                bond_all_to_central(atoms_count, &mut molecule, central_atom.clone(), BondType::PI);
            }

        }
    }

    molecule
}
