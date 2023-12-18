mod sparse_set;

use std::fmt::Display;
use comfy_table::Table;
use sparse_set::SparseSet;

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

struct DancingCells {
    itm: Vec<usize>,
    clr: Vec<Option<usize>>,
    loc: Vec<usize>,
    set: Vec<usize>,
    item: Vec<usize>,
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
        let mut loc = loc.chars().map(|c| c.to_digit(10).map(|n| n as usize)).collect::<Vec<_>>();

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

        {
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
            }
        }

        let loc = loc.iter().map(|&x| x.unwrap()).collect::<Vec<_>>();
        let set = set.iter().map(|&x| x.unwrap()).collect::<Vec<_>>();
        let itm = itm.iter().map(|&x| x.unwrap()).collect::<Vec<_>>();

        Self { itm, clr, loc, set, item }
    }
}

impl Display for DancingCells {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = Table::new();
        let mut row = vec!["x".to_owned()];
        row.extend((0..self.itm.len()).map(|i| format!("{:2}", i)));
        table.add_row(row);
        let mut row = vec!["ITM[x]".to_owned()];
        row.extend(self.itm.iter().map(|&i| format!("{:2}", i)));
        table.add_row(row);
        let mut row = vec!["CLR[x]".to_owned()];
        row.extend(self.clr.iter().map(|&i| match i {
            None => "  ".to_owned(),
            Some(i) => format!("{:2}", i),
        }));
        table.add_row(row);
        let mut row = vec!["LOC[x]".to_owned()];
        row.extend(self.loc.iter().map(|&i| format!("{:2}", i)));
        table.add_row(row);
        println!("{table}");

        let mut table = Table::new();
        table.add_row(vec!["i", "SET[i]"]);
        {
            let mut i = 0;
            loop {
                table.add_row(vec!["POS".to_owned(), format!("{:2}", self.set[i])]);
                table.add_row(vec!["SIZE".to_owned(), format!("{:2}", self.set[i+1])]);
                for i in 0..self.set[i+1] {
                    table.add_row(vec!["".to_owned(), format!("{:2}", self.set[i+2+i])]);
                }
                i += self.set[i+1] + 2;
                if i >= self.set.len() {
                    break;
                }
            }
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

    let dc = DancingCells::from_str("pqrxy", "ABC", 
        " pqxy prxy px qx ry ", 
        "   CA   AC  B  A  B ", 
        "4    4    2  2  2  0"
    );
    println!("{dc}");
}
