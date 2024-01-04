#![doc = include_str!("../README.md")]
#![doc(issue_tracker_base_url = "https://github.com/recmo/dancing-cells/issues/")]

mod sparse_set;

use comfy_table::Table;
use sparse_set::SparseSet;
use std::{fmt::{Debug, Display}, num::ParseIntError};

/// Exact Cover with Colors (XCC)
/// Primary items must be covered by exactly one option.
/// Secondary items must be covered by the same color.
struct ExactCoverProblem {
    num_primary: usize,
    num_secondary: usize,
    options: Vec<Covering>,
}

struct Covering {
    primary: Vec<usize>,
    secondary: Vec<(usize, usize)>,
}

#[derive(Clone, PartialEq, Eq)]
struct DancingCells {
    itm: Vec<usize>,
    clr: Vec<Option<usize>>,
    loc: Vec<usize>,
    sgn: Vec<bool>,
    set: Vec<usize>,
    item: Vec<usize>,
    second: usize,
    active: usize,
    oactive: usize,
    flag: bool,
    trail: Vec<(usize, usize)>,
    solution: Vec<usize>,
    indenation: String,
}

impl DancingCells {
    pub fn from_str(symbols: &str, colors: &str, itm: &str, clr: &str, loc: &str) -> Self {
        assert_eq!(itm.len(), clr.len());
        assert_eq!(itm.len(), loc.len());
        assert!(itm.chars().all(|c| c == ' ' || symbols.contains(c)));
        assert!(clr.chars().all(|c| c == ' ' || colors.contains(c)));
        assert!(loc.chars().all(|c| c == ' ' || c.is_ascii_digit()));

        let mut itm = itm.chars().map(|c| symbols.find(c)).collect::<Vec<_>>();
        let clr = clr.chars().map(|c| colors.find(c)).collect::<Vec<_>>();
        let mut loc = loc
            .chars()
            .map(|c| c.to_digit(10).map(|n| n as usize))
            .collect::<Vec<_>>();

        let mut set = vec![];
        let mut item = vec![];
        for symbol in 0..symbols.len() {
            set.push(Some(item.len())); // POS
            let size_index = set.len();
            set.push(None); // SIZE
            item.push(set.len());
            let mut size = 0;
            for i in itm
                .iter()
                .enumerate()
                .filter(|(_, &s)| s == Some(symbol))
                .map(|(i, _)| i)
            {
                loc[i] = Some(set.len());
                set.push(Some(i));
                size += 1;
            }
            set[size_index] = Some(size);
        }

        for itm in itm.iter_mut() {
            if let Some(itm) = itm {
                *itm = item[*itm];
            }
        }

        let mut sgn = vec![false; loc.len()];
        {
            sgn[0] = true;
            itm[0] = Some(0);
            let mut i = 0;
            loop {
                let length = loc[i].unwrap();
                if length == 0 {
                    break;
                }
                i += length + 1;
                assert!(i < loc.len());
                itm[i] = Some(length);
                sgn[i] = true;
            }
        }

        let loc = loc.iter().map(|&x| x.unwrap()).collect::<Vec<_>>();
        let set = set.iter().map(|&x| x.unwrap()).collect::<Vec<_>>();
        let itm = itm.iter().map(|&x| x.unwrap()).collect::<Vec<_>>();

        let second = 15; // TODO
        let active = item.len();
        let oactive = active;
        let flag = false;
        Self {
            itm,
            clr,
            loc,
            set,
            sgn,
            item,
            second,
            active,
            oactive,
            flag,
            trail: vec![],
            solution: vec![],
            indenation: "".to_owned(),
        }
    }

    pub fn check_consistency(&self) {
        assert_eq!(self.itm.len(), self.clr.len());
        assert_eq!(self.itm.len(), self.loc.len());

        // Pos and item are inverse permutations.
        for k in 0..self.item.len() {
            assert_eq!(self.set[self.item[k] - 2], k, "k = {}", k);
        }

        // Loc and set are inverse permutations.
        for item in self.item.iter().copied() {
            let size = self.size(item);
            for i in item..item + size {
                assert_eq!(self.loc[self.set[i]], i);
            }
        }
    }

    /// Finds an option to try, or [`None`] if stuck.
    /// Iterates through all items and picks the first option from the item
    /// with fewest options.
    fn select(&self) -> Option<usize> {
        // Find item with fewest options.
        let item = self.iter_items()
            .take(self.active)
            .min_by_key(|&i| self.size(i))?;

        // Pick the first option from the item.
        let mut option = self.set[item..item + self.size(item)].iter().copied().next()?;

        // Reduce to the first entry of the option.
        while !self.sgn[option] {
            option += 1;
        }
        option -= self.itm[option];
        Some(option)
    }

    fn is_solved(&self) -> bool {
        self.active == 0
    }

    fn is_stuck(&self) -> bool {
        self.iter_items()
            .take(self.active)
            .map(|i| self.size(i))
            .any(|size| size == 0)
    }

    /// Recursive solver
    fn solver(&mut self) {
        eprintln!("{}Solver", self.indenation);
        // dbg!(&self);
        self.check_consistency();
        
        // Find an item to try all options of.
        let Some(item) = self.iter_items()
            .take(self.active)
            .min_by_key(|&i| self.size(i)) else {
                // No items left to try, we are done.
                eprintln!("{}Solution: {:?}", self.indenation, self.solution);
                return;
            };
        let size = self.size(item);
        if size == 0 {
            // We are stuck, backtrack.
            eprintln!("{}Stuck", self.indenation);
            return;
        }        
        let options = self.set[item..item + self.size(item)].iter().copied().collect::<Vec<_>>();
        eprintln!("{}Exploring item {item} with options {options:?}", self.indenation);
        self.indenation.push_str("  ");

        for option in options {
            // dbg!(&self);
            eprintln!("{}Trying option {option}", self.indenation);
            self.solution.push(option);
            self.indenation.push_str("  ");

            // Store sizes
            let old_active = self.active;
            let mut old_sizes = self.item[..self.active].iter().map(|&i| self.size(i)).collect::<Vec<_>>();

            // Iterate through the option's items.
            let size = self.loc[option - 1];
            for i in option..option + size {
                // Hide the item and all options that contain it.
                let item = self.itm[i];
                let color = self.clr[i];
                let k = self.pos(item);

                // Skip if already hidden.
                if k >= self.active {
                    continue;
                }

                // Hide from the item list.
                assert!(k < self.active);
                self.active -= 1;
                self.swap_item(k, self.active);
                old_sizes.swap(k, self.active);

                // Hide all options that contain it.
                self.hide(item, color);
            }

            // Recurse
            self.solver();

            // Restore sizes
            eprintln!("{}Reverting option {option}", self.indenation);
            self.active = old_active;
            self.item[..self.active].iter().zip(old_sizes).for_each(|(&i, size)| self.set[i - 1] = size);
            self.solution.pop();
            self.indenation.pop();
            self.indenation.pop();
        }

        eprintln!("{}Backtracking item {item}", self.indenation);
        self.indenation.pop();
        self.indenation.pop();

        self.check_consistency();
    }

    fn solve(&mut self) {
        dbg!(self.is_stuck());

        // C2 Pick an option i to try.
        // TODO: Method from knuth
        let k = 0;
        let i = self.item[k];
        
        // C3 Deactivate k
        self.remove_item(k);

        // C4 Hide i
        self.oactive = self.active;
        self.flag = false;
        self.hide(i, None);

        // C5 Trail the sizes.
        let max_trail = 1000;
        if self.trail.len() + self.active > max_trail {}
        self.trail.push((i, self.size(i)));
    }

    fn apply(&mut self) {
        
    }

    fn backtrack(&mut self) {

    }

    /// Hide an item and all options that contain it.
    fn hide(&mut self, item: usize, color: Option<usize>) {
        eprintln!("{}Hiding item {item} colour {color:?}", self.indenation);
        assert!(self.item.contains(&item));
        let size = self.size(item);

        // Iterate through all options.
        for i in item..item + size {
            let x = self.set[i];

            // Skip options with compatible colors
            if color.is_some() && self.clr[x] == color {
                continue;
            }

            // Find siblings items of the option.
            let mut xi = x;
            loop {
                // Advance index, looping around
                xi += 1;
                if self.sgn[xi] {
                    xi -= self.itm[xi];
                }
                // Stop if we reached the initial option again
                if xi == x {
                    break;
                }

                // Remove the option from the item.
                let ii = self.itm[xi];

                // Skip if item is not active.
                if self.pos(ii) >= self.oactive {
                    continue;
                }

                // Early exit if it is the last option, there is no solution.
                // if self.size(ii) == 1
                //     && self.flag == false
                //     && ii < self.second
                //     && self.pos(ii) < self.active
                // {
                //     self.flag = true;
                //     dbg!();
                //     return;
                // }

                let loc = self.loc[xi];
                self.remove_option(ii, loc);
            }
        }
    }

    fn pos(&self, item: usize) -> usize {
        assert!(self.item.contains(&item));
        self.set[item - 2]
    }

    fn size(&self, item: usize) -> usize {
        assert!(self.item.contains(&item));
        self.set[item - 1]
    }

    fn iter_set(&self, item: usize) -> impl Iterator<Item = usize> + '_ {
        assert!(self.item.contains(&item));
        self.set[item..item + self.size(item)].iter().copied()
    }

    fn iter_items(&self) -> impl Iterator<Item = usize> + '_ {
        self.item[..self.active].iter().copied()
    }

    fn remove_item(&mut self, k: usize) {
        assert!(k < self.active);
        self.active -= 1;
        self.swap_item(k, self.active);
    }

    fn remove_option(&mut self, item: usize, k: usize) {
        assert!(self.item.contains(&item));
        assert!(k >= item);
        let end = item + self.size(item);
        assert!(k < end);
        self.swap_option(item, k, end - 1);
        self.set[item - 1] -= 1; // Reduce size
    }

    fn unremove_item(&mut self) {
        self.active += 1;
    }

    fn unremove_option(&mut self, item: usize) {
        assert!(self.item.contains(&item));
        self.set[item - 1] += 1; // Increase size
    }

    /// Swap item entries in item preserving invariants.
    fn swap_item(&mut self, i: usize, j: usize) {
        assert!(i < self.item.len());
        assert!(j < self.item.len());
        self.item.swap(i, j);
        self.set.swap(self.item[i] - 2, self.item[j] - 2);
    }

    /// Swap option entries in set preserving invariants.
    fn swap_option(&mut self, item: usize, i: usize, j: usize) {
        assert!(self.item.contains(&item));
        let end = item + self.size(item);
        let range = item..end;
        assert!(range.contains(&i));
        assert!(range.contains(&j));
        self.set.swap(i, j);
        self.loc.swap(self.set[i], self.set[j]);
    }
}

impl Debug for DancingCells {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n{}", self)
    }
}

impl Display for DancingCells {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = Table::new();
        let mut row = vec!["x".to_owned()];
        row.extend((0..self.itm.len()).map(|i| format!("{:2}", i)));
        table.add_row(row);
        let mut row = vec!["ITM[x]".to_owned()];
        row.extend(
            self.itm
                .iter()
                .enumerate()
                .map(|(i, &j)| format!("{}{j:2}", if self.sgn[i] { "-" } else { "" })),
        );
        table.add_row(row);
        let mut row = vec!["CLR[x]".to_owned()];
        row.extend(self.clr.iter().map(|&i| match i {
            None => "  ".to_owned(),
            Some(i) => format!("{:2}", i),
        }));
        table.add_row(row);
        let mut row = vec!["LOC[x]".to_owned()];
        row.extend(
            self.loc
                .iter()
                .enumerate()
                .map(|(i, &j)| format!("{}{j:2}", if self.sgn[i] { "+" } else { "" })),
        );
        table.add_row(row);
        writeln!(f, "{table}")?;

        let mut table = Table::new();
        let mut row = vec!["k".to_owned()];
        row.extend((0..self.item.len()).map(|i| format!("{:2}", i)));
        table.add_row(row);
        let mut row = vec!["ITEM[k]".to_owned()];
        row.extend(self.item.iter().map(|&i| format!("{:2}", i)));
        table.add_row(row);
        writeln!(f, "{table}")?;

        writeln!(f, "SECOND = {}", self.second)?;
        writeln!(f, "ACTIVE = {}", self.active)?;

        let mut table = Table::new();
        table.add_row(vec!["", "i", "SET[i]"]);
        for i in 0..self.set.len() {
            let label = if self.item.contains(&(i + 2)) {
                "POS"
            } else if self.item.contains(&(i + 1)) {
                "SIZE"
            } else {
                ""
            };
            table.add_row(vec![
                label.to_owned(),
                format!("{}", i),
                format!("{:2}", self.set[i]),
            ]);
        }
        write!(f, "{table}")
    }
}

fn main() {
    let mut ss = SparseSet::new(10);
    println!("ss: {:?}", ss);
    println!("ss: {:?}", ss.iter());
    for i in [2, 5, 3, 7] {
        ss.remove(i);
    }
    println!("ss: {:?}", ss);
    println!("ss: {:?}", ss.iter());
    ss.undo();
    println!("ss: {:?}", ss.iter());
    ss.undo();
    println!("ss: {:?}", ss.iter());

    let problem = ExactCoverProblem {
        num_primary: 3,   // p q r
        num_secondary: 2, // x y
        options: vec![
            Covering {
                primary: vec![0, 1],             // p q
                secondary: vec![(0, 2), (1, 0)], // x:* y:A
            },
            Covering {
                primary: vec![0, 2],             // p r
                secondary: vec![(0, 0), (1, 3)], // x:A y:*
            },
            Covering {
                primary: vec![0],        // p
                secondary: vec![(0, 1)], // x: B
            },
            Covering {
                primary: vec![1],        // q
                secondary: vec![(0, 0)], // x: A
            },
            Covering {
                primary: vec![2],        // r
                secondary: vec![(1, 1)], // y: B
            },
        ],
    };

    let mut dc = DancingCells::from_str(
        "pqrxy",
        "ABC",
        " pqxy prxy px qx ry ",
        "   CA   AC  B  A  B ",
        "4    4    2  2  2  0",
    );
    println!("{dc}");

    dc.check_consistency();

    dc.solver();

    dc.check_consistency();
}

