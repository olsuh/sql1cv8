use hashbrown::HashMap;
use regex::Regex;
use std::error::Error;

use crate::Metadata;

// Parse преобразует в тексте имена метаданных в имена базы данных.
// В качестве параметров принимает строковую переменную:
// src - текст запроса.
// Возвращает изменённый запрос res.
impl Metadata {
    pub fn parse(&self, src: &str) -> Result<String, Box<dyn Error>> {
        let mut buf = Vec::new();
        let mut res = src.to_string();

        res = remove_strings_and_comments(&res, &mut buf);
        res = mark_statements(&res);
        res = parse_func_constructions(self, &res);
        res = parse_full_constructions(self, &res);
        res = parse_with_brackets(self, &res)?;
        res = restore_strings_and_comments(&res, &buf);

        Ok(res)
    }
}

fn parse_func_constructions(m: &Metadata, src: &str) -> String {
    let mut res = src.to_string();

    // UUID tables
    let re = Regex::new(r"\[\$([\pL\w\.]+)\]\.UUID").unwrap();
    res = re
        .replace_all(&res, |s: &regex::Captures| {
            let tabname = &s[1];
            if let Some(table_object) = m.objects.get(tabname) {
                format!("'{}'", table_object.uuid)
            } else {
                s[0].to_string()
            }
        })
        .to_string();

    // UUID fields
    let re = Regex::new(r"\[\$([\pL\w\.]+)\]\.\[\$([\pL\w\.]+)\]\.UUID").unwrap();
    res = re
        .replace_all(&res, |s: &regex::Captures| {
            let tabname = &s[1];
            let colname = &s[2];
            if let Some(table_object) = m.objects.get(tabname) {
                if let Some(field_object) = table_object.params.get(colname) {
                    format!("'{}'", field_object.uuid)
                } else {
                    s[0].to_string()
                }
            } else {
                s[0].to_string()
            }
        })
        .to_string();

    // Type tables
    let re = Regex::new(r"\[\$([\pL\w\.]+)\]\.Type").unwrap();
    res = re
        .replace_all(&res, |s: &regex::Captures| {
            let tabname = &s[1];
            if let Some(table_object) = m.objects.get(tabname) {
                format!("'{}'", table_object.r#type)
            } else {
                s[0].to_string()
            }
        })
        .to_string();

    // Type fields
    let re = Regex::new(r"\[\$([\pL\w\.]+)\]\.\[\$([\pL\w\.]+)\]\.Type").unwrap();
    res = re
        .replace_all(&res, |s: &regex::Captures| {
            let tabname = &s[1];
            let colname = &s[2];
            if let Some(table_object) = m.objects.get(tabname) {
                if let Some(field_object) = table_object.params.get(colname) {
                    format!("'{}'", field_object.r#type)
                } else {
                    s[0].to_string()
                }
            } else {
                s[0].to_string()
            }
        })
        .to_string();

    // Number tables
    let re = Regex::new(r"\[\$([\pL\w\.]+)\]\.Number").unwrap();
    res = re
        .replace_all(&res, |s: &regex::Captures| {
            let tabname = &s[1];
            if let Some(table_object) = m.objects.get(tabname) {
                table_object.number.clone()
            } else {
                s[0].to_string()
            }
        })
        .to_string();

    // Number fields
    let re = Regex::new(r"\[\$([\pL\w\.]+)\]\.\[\$([\pL\w\.]+)\]\.Number").unwrap();
    res = re
        .replace_all(&res, |s: &regex::Captures| {
            let tabname = &s[1];
            let colname = &s[2];
            if let Some(table_object) = m.objects.get(tabname) {
                if let Some(field_object) = table_object.params.get(colname) {
                    field_object.number.clone()
                } else {
                    s[0].to_string()
                }
            } else {
                s[0].to_string()
            }
        })
        .to_string();

    // DBName tables
    let re = Regex::new(r"\[\$([\pL\w\.]+)\]\.DBName").unwrap();
    res = re
        .replace_all(&res, |s: &regex::Captures| {
            let tabname = &s[1];
            if let Some(table_object) = m.objects.get(tabname) {
                format!("'{}'", table_object.db_name)
            } else {
                s[0].to_string()
            }
        })
        .to_string();

    // DBName fields
    let re = Regex::new(r"\[\$([\pL\w\.]+)\]\.\[\$([\pL\w\.]+)\]\.DBName").unwrap();
    res = re
        .replace_all(&res, |s: &regex::Captures| {
            let tabname = &s[1];
            let colname = &s[2];
            if let Some(table_object) = m.objects.get(tabname) {
                if let Some(field_object) = table_object.params.get(colname) {
                    format!("'{}'", field_object.db_name)
                } else {
                    s[0].to_string()
                }
            } else {
                s[0].to_string()
            }
        })
        .to_string();

    res
}

fn parse_with_brackets(m: &Metadata, src: &str) -> Result<String, Box<dyn Error>> {
    let mut res = String::new();
    let mut open = 0;
    let mut inc = String::new();
    let mut src = src.to_string();

    loop {
        let i = src.find("(");
        let j = src.find(")");
        if i.is_none() && j.is_none() {
            if open > 0 {
                return Err("не закрыта скобка".into());
            }
            res += &src;
            break;
        }
        let i = i.unwrap_or(j.unwrap() + 1);
        let j = j.unwrap_or(i + 1);
        if i < j {
            if open == 0 {
                res += &src[..=i];
                inc.clear();
            } else {
                inc += &src[..=i];
            }
            src = src[i + 1..].to_string();
            open += 1;
        } else {
            if open == 0 {
                return Err("ошибочное закрытие скобки".into());
            }
            open -= 1;
            if open == 0 {
                inc += &src[..j];
                let s = parse_with_brackets(m, &inc)?;
                res += &format!("{}{})", s, ")");
                src = src[j + 1..].to_string();
            } else {
                inc += &src[..=j];
                src = src[j + 1..].to_string();
            }
        }
    }

    let re = Regex::new(r"▶[^▶]+").unwrap();
    res = re
        .replace_all(&res, |s: &regex::Captures| parse_with_aliases(m, &s[1]))
        .to_string();
    res = unmark_statements(&res);

    Ok(res)
}

fn mark_statements(src: &str) -> String {
    let mut src = format!("▶{}", src);
    let re = Regex::new(r"(?si)\b((?:select|bulk|insert|update|delete|merge)\s)").unwrap();
    src = re.replace_all(&src, r"▶$1").to_string();
    src
}

fn unmark_statements(src: &str) -> String {
    src.replace("▶", "")
}

fn parse_full_constructions(m: &Metadata, src: &str) -> String {
    let re = Regex::new(r"\[\$([\pL\w\.]+)\]\.\[\$([\pL\w\.]+)\]").unwrap();
    re.replace_all(src, |s: &regex::Captures| {
        let tabname = &s[1];
        let colname = &s[2];
        if let Some(table_object) = m.objects.get(tabname) {
            if let Some(field_object) = table_object.params.get(colname) {
                format!("{}.{}", table_object.db_name, field_object.db_name)
            } else {
                s[0].to_string()
            }
        } else {
            s[0].to_string()
        }
    })
    .to_string()
}

fn parse_with_aliases(m: &Metadata, src: &str) -> String {
    let mut res = src.to_string();
    let mut aliases = HashMap::new();

    let re = Regex::new(r"(?si)(?:\.\.|\[dbo\]\.|\bdbo\.|[^\.])\[\$(\$?[\pL\w\.]+)\](?:\s+as\s+|\s+)(?:\[(.+?)\]|(\w+))").unwrap();
    for v in re.captures_iter(&res) {
        let tabname = &v[1];
        let aliasname = format!("{}{}", &v[2], &v[3]);
        aliases.insert(aliasname, tabname.to_string());
    }

    let re = Regex::new(r"(?si)((?:\.\.|\[dbo\]\.|\bdbo\.|[^\.]))\[\$(\$?[\pL\w\.]+)\]").unwrap();
    res = re
        .replace_all(&res, |s: &regex::Captures| {
            let prefix = &s[1];
            let tabname = &s[2];
            if let Some(table_object) = m.objects.get(tabname) {
                format!("{}{}", prefix, table_object.db_name)
            } else {
                s[0].to_string()
            }
        })
        .to_string();

    let re = Regex::new(r"(?si)((?:\[(.+?)\]|(\w+))\.)\[\$([\pL\w\.]+)\]").unwrap();
    res = re
        .replace_all(&res, |s: &regex::Captures| {
            let prefix = &s[1];
            let aliasname = format!("{}{}", &s[2], &s[3]);
            let colname = &s[4];
            if let Some(tabname) = aliases.get(&aliasname) {
                if let Some(table_object) = m.objects.get(tabname) {
                    if let Some(field_object) = table_object.params.get(colname) {
                        format!("{}{}", prefix, field_object.db_name)
                    } else {
                        s[0].to_string()
                    }
                } else {
                    s[0].to_string()
                }
            } else {
                s[0].to_string()
            }
        })
        .to_string();

    res
}

fn restore_strings_and_comments(src: &str, buf: &[String]) -> String {
    let mut res = src.to_string();
    for (i, old) in buf.iter().enumerate().rev() {
        let new = format!("«{}»", i);
        res = res.replacen(&new, old, 1);
    }
    res
}

fn remove_strings_and_comments(src: &str, buf: &mut Vec<String>) -> String {
    let mut open = 0;
    let mut res = String::new();
    let mut src = src;
    let mut clr = "";
    let mut sub = "";

    loop {
        if open == 0 {
            let i1 = src.find("/*");
            let i2 = src.find("--");
            let i3 = src.find("'");
            let i4 = src.find("\"");
            let i = min(&[i1, i2, i3, i4], src.len());
            if i == src.len() {
                res += &src;
                break;
            }
            let com;
            (com, sub, clr) = match i {
                i if i == i1.unwrap() => ("/*", "/*", "*/"),
                i if i == i2.unwrap() => ("--", "", "\n"),
                i if i == i3.unwrap() => ("'", "", "'"),
                i if i == i4.unwrap() => ("\"", "", "\""),
                _ => unreachable!(),
            };
            res += &src[..i];
            src = &src[i + com.len()..];
            open += 1;
        } else {
            let i = find_and_len(src, sub);
            let j = find_and_len(src, clr);
            let k = min(&[i, j, Some(src.len())], src.len());
            if k == src.len() {
                buf.push(src.to_string());
                res += &format!("«{}»", buf.len() - 1);
                break;
            }
            let com = &src[..k];
            src = &src[k..];
            if i < j {
                open += 1;
            } else {
                open -= 1;
                if open == 0 {
                    buf.push(com.to_string());
                    res += &format!("«{}»", buf.len() - 1);
                }
            }
        }
    }
    res
}

/*fn index(s: &str, substr: &str) -> usize {
    if substr.is_empty() {
        return s.len();
    }
    s.find(substr).unwrap_or(s.len())
}*/
fn find_and_len(src: &str, sub_str: &str) -> Option<usize> {
    src.find(sub_str).and_then(|x| Some(x + sub_str.len()))
}

fn min(i: &[Option<usize>], start: usize) -> usize {
    let mut r = start;
    for &v in i {
        if let Some(v) = v {
            if r > v {
                r = v;
            }
        }
    }
    r
}
