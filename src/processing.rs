use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct CVNames {
    pub(crate) m: HashMap<String, CVName>,
}

#[derive(Default)]
struct CVName {
    pub(crate) val: String,
    pub(crate) syn: HashMap<String, String>,
}

fn processing_cv_names(bin: &[u8]) -> CVNames {
    let mut c = CVNames {
        m: HashMap::with_capacity(65536),
    };

    let mut i = String::new();
    let mut l = String::new();
    let mut pd = processing(bin);

    while pd.next() {
        let (level, posit, _) = pd.get();
        if level == 1 && posit == 1 {
            break;
        }
    }

    while pd.next() {
        let (level, posit, value) = pd.get();
        match level {
            2 => match posit % 7 {
                1 => i = value,
                4 => {
                    c.m.insert(
                        i.clone(),
                        CVName {
                            val: value,
                            syn: HashMap::new(),
                        },
                    );
                }
                _ => {}
            },
            4 => match posit {
                0 => l = value,
                1 => {
                    if let Some(cv_name) = c.m.get_mut(&i) {
                        cv_name.syn.insert(l.clone(), value);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    c
}

pub(crate) type Enums = Vec<Enum>;

pub(crate) struct Enum {
    pub(crate) num: String,
    pub(crate) val: String,
    pub(crate) syn: HashMap<String, String>,
}

fn processing_enums(bin: &[u8]) -> Enums {
    let mut es = Vec::with_capacity(16);
    let mut i = 0;
    let mut l = String::new();
    let mut y = false;
    let mut e: Option<Enum> = None;
    let mut pd = processing(bin);

    while pd.next() {
        let (level, posit, _) = pd.get();
        if level == 1 && posit == 5 {
            break;
        }
    }

    while pd.next() {
        let (level, posit, value) = pd.get();
        match level {
            5 => {
                y = false;
                if posit == 2 {
                    e = Some(Enum {
                        num: i.to_string(),
                        val: value,
                        syn: HashMap::with_capacity(1),
                    });
                    es.push(e.unwrap());
                    i += 1;
                    y = true;
                }
            }
            6 => {
                if !y || posit == 0 {
                    continue;
                }
                match posit % 2 {
                    0 => {
                        if let Some(enum_ref) = es.last_mut() {
                            enum_ref.syn.insert(l.clone(), value);
                        }
                    }
                    1 => l = value,
                    _ => {}
                }
            }
            _ => {}
        }
    }

    es
}

#[derive(Debug, Default)]
pub(crate) struct DBNames {
    pub(crate) m: HashMap<String, DBName>,
    cnt_enums: i32,
    qry_enums: String,
    cnt_points: i32,
    qry_points: String,
}

#[derive(Debug, Default)]
pub(crate) struct DBName {
    pub(crate) ids: String,
    pub(crate) typ: String,
    pub(crate) num: String,
}

fn processing_db_names(bin: &[u8]) -> DBNames {
    let mut d = DBNames {
        m: HashMap::with_capacity(65536),
        cnt_enums: 0,
        qry_enums: String::new(),
        cnt_points: 0,
        qry_points: String::new(),
    };

    let mut i = String::new();
    let mut t = String::new();
    let mut n = String::new();
    let mut ce = 0;
    let mut cp = 0;
    let mut qe = String::new();
    let mut qp = String::new();

    let mut pd = processing(bin);

    while pd.next() {
        let (level, posit, value) = pd.get();
        if level == 3 {
            match posit {
                0 => i = value,
                1 => t = value,
                2 => {
                    n = value;
                    d.m.insert(
                        t.clone() + &n,
                        DBName {
                            ids: i.clone(),
                            typ: t.clone(),
                            num: n.clone(),
                        },
                    );

                    match t.as_str() {
                        "Enum" => {
                            ce += 1;
                            qe.push_str(&format!(",'{}'", i));
                        }
                        "BPrPoints" => {
                            cp += 1;
                            qp.push_str(&format!(",'{}.7'", i));
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    if !qe.is_empty() {
        d.cnt_enums = ce;
        d.qry_enums = format!(
            "select FileName, BinaryData from Config where FileName in ({})",
            &qe[1..]
        );
    }
    if !qp.is_empty() {
        d.cnt_points = cp;
        d.qry_points = format!(
            "select left(FileName, 36) FileName, BinaryData from Config where FileName in ({})",
            &qp[1..]
        );
    }

    d
}

struct ProcessingData<'a> {
    bin: &'a [u8],
    current_level: i32,
    current_posit: Vec<i32>,
    last_level: i32,
    last_posit: i32,
    last_value: Vec<u8>,
}

fn processing(bin: &[u8]) -> ProcessingData {
    ProcessingData {
        bin: &bin[3..],
        current_level: 0,
        current_posit: vec![0; 64],
        last_level: 0,
        last_posit: 0,
        last_value: Vec::with_capacity(256),
    }
}

impl<'a> ProcessingData<'a> {
    fn get(&self) -> (i32, i32, String) {
        (
            self.last_level,
            self.last_posit,
            String::from_utf8_lossy(&self.last_value).to_string(),
        )
    }

    fn next(&mut self) -> bool {
        self.last_value.clear();
        let mut is_string = false;
        for (i, &v) in self.bin.iter().enumerate() {
            match v {
                123 => {
                    // {
                    self.current_level += 1;
                    self.current_posit[self.current_level as usize] = 0;
                    continue;
                }
                125 => {
                    // }
                    self.bin = &self.bin[i + 1..];
                    self.last_level = self.current_level;
                    self.last_posit = self.current_posit[self.current_level as usize];
                    self.current_level -= 1;
                    return true;
                }
                44 => {
                    // ,
                    self.bin = &self.bin[i + 1..];
                    self.last_level = self.current_level;
                    self.last_posit = self.current_posit[self.current_level as usize];
                    self.current_posit[self.current_level as usize] += 1;
                    return true;
                }
                34 => {
                    // "
                    if is_string && self.bin.get(i + 1) == Some(&34) {
                        is_string = false;
                    } else {
                        is_string = !is_string;
                        continue;
                    }
                }
                10 | 13 => {
                    if !is_string {
                        continue;
                    }
                }
                _ => {}
            }
            self.last_value.push(v);
        }
        false
    }
}

pub(crate) type Points = Vec<Point>;

pub(crate) type Point = Enum;
/*struct Point {
    pub(crate) num: String,
    pub(crate) val: String,
    pub(crate) syn: HashMap<String, String>,
}*/

fn processing_points(bin: &[u8]) -> Points {
    let mut ps = Vec::with_capacity(16);
    let mut n = String::new();
    let mut l = String::new();
    let mut y = false;
    let mut p: Option<Point> = None;
    let mut s: HashMap<String, String> = HashMap::new();
    let mut pd = processing(bin);

    while pd.next() {
        let (level, posit, _) = pd.get();
        if level == 1 && posit == 3 {
            break;
        }
    }

    while pd.next() {
        let (level, posit, value) = pd.get();
        match level {
            3 => {
                if y {
                    while pd.next() {
                        let (level, _, _) = pd.get();
                        if level == 1 {
                            break;
                        }
                    }
                    y = false;
                }
            }
            4 => match posit {
                0 => {
                    s = HashMap::with_capacity(1);
                    y = true;
                }
                3 => n = value,
                4 => {
                    p = Some(Point {
                        num: value,
                        val: n.clone(),
                        syn: s.clone(),
                    });
                    ps.push(p.unwrap());
                }
                _ => {}
            },
            6 => match posit {
                0 => l = value,
                1 => {
                    s.insert(l.clone(), value);
                }
                _ => {}
            },
            _ => {}
        }
    }

    ps
}
