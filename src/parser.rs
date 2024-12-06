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
                res += &(s + ")");
                src = src[j + 1..].to_string();
            } else {
                inc += &src[..=j];
                src = src[j + 1..].to_string();
            }
        }
    }

    let re = Regex::new(r"▶[^▶]+").unwrap();
    let res = re.replace_all(&res, |s: &regex::Captures| parse_with_aliases(m, &s[0]));
    let res = unmark_statements(&res);

    Ok(res)
}

fn mark_statements(src: &str) -> String {
    let mut src = format!("▶{}", src);
    let re = Regex::new(r#"(?si)\b((?:select|bulk|insert|update|delete|merge)\s)"#).unwrap();
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

trait MyIndex<I> {
    //type Output = str;

    fn index<'a>(&'a self, i: I) -> &'a str;
}

use regex::Captures;
impl<'h> MyIndex<usize> for Captures<'h> {
    //type Output = str;

    // The lifetime is written out to make it clear that the &str returned
    // does NOT have a lifetime equivalent to 'h.
    fn index<'a>(&'a self, i: usize) -> &'a str {
        self.get(i).map(|m| m.as_str()).unwrap_or_else(|| "")
    }
}

fn parse_with_aliases(m: &Metadata, src: &str) -> String {
    let mut res = src.to_string();
    let mut aliases = HashMap::new();

    let re = Regex::new(r"(?si)(?:\.\.|\[dbo\]\.|\bdbo\.|[^\.])\[\$(\$?[\pL\w\.]+)\](?:\s+as\s+|\s+)(?:\[(.+?)\]|(\w+))").unwrap();
    for v in re.captures_iter(&res) {
        let tabname = &v[1];
        let aliasname = format!("{}{}", v.index(2), &v[3]);
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
            let aliasname = format!("{}{}", s.index(2), &s[3]);
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
    let mut end = "";
    let mut sub = "";
    let mut com_len = 0;
    let mut point = 0;
    let mut search;

    loop {
        search = &src[point..];
        if open == 0 {
            let i1 = search.find("/*");
            let i2 = search.find("--");
            let i3 = search.find("'");
            let i4 = search.find("\"");
            let point_i = min(&[i1, i2, i3, i4], search.len());
            res += &search[..point_i];
            if point_i == search.len() {
                break;
            }

            (com_len, sub, end) = match Some(point_i) {
                i if i == i1 => ("/*".len(), "/*", "*/"),
                i if i == i2 => ("--".len(), "", "\n"),
                i if i == i3 => ("'".len(), "", "'"),
                i if i == i4 => ("\"".len(), "", "\""),
                _ => unreachable!(),
            };
            point += point_i + com_len;
            open += 1;
        } else {
            let i_sub = find_after(search, sub);
            let i_end = find_after(search, end);
            let point_i = min(&[i_sub, i_end], search.len());
            let com = &src[point - com_len..point + point_i];
            if point_i == search.len() {
                res += &format!("«{}»", buf.len());
                buf.push(com.to_string());
                break;
            }
            let is_open = matches!((i_sub,i_end), (Some(i_s), Some(i_e)) if i_s < i_e);
            if is_open {
                open += 1;
            } else {
                open -= 1;
                if open == 0 {
                    res += &format!("«{}»", buf.len());
                    buf.push(com.to_string());
                }
            }
            com_len += point_i;
            point += point_i;
        }
    }
    res
}

fn find_after(src: &str, sub_str: &str) -> Option<usize> {
    if sub_str.is_empty() {
        None
    } else {
        src.find(sub_str).and_then(|pos| Some(pos + sub_str.len()))
    }
}

fn min(arr: &[Option<usize>], max: usize) -> usize {
    let mut min = max;
    for &i in arr {
        if let Some(i) = i {
            if min > i {
                min = i;
            }
        }
    }
    min
}
