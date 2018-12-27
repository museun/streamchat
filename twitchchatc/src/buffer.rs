use super::*;

use termcolor::{Color, ColorSpec, WriteColor};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub(crate) struct Buffer<'a> {
    opts: &'a Options,
}

impl<'a> Buffer<'a> {
    pub fn format(opts: &'a Options, msg: &'a Message, buf: &mut termcolor::Buffer) {
        Self { opts }.print(msg, buf)
    }

    fn print(self, msg: &'a Message, buf: &mut termcolor::Buffer) {
        self.add_name(buf, &msg);
        let lines = self.split_line(&msg.data);
        self.write_lines(buf, &lines);
    }

    fn add_name(&self, buf: &mut termcolor::Buffer, msg: &'a Message) {
        let mut name = msg.name.clone();
        truncate(&mut name, self.opts.name_max);
        let pad = self.opts.name_max.saturating_sub(name.width()) + 1;

        if msg.is_action {
            write!(buf, "{:>offset$}", "*", offset = pad).unwrap();
        } else {
            write!(buf, "{:>offset$}", " ", offset = pad).unwrap();
        }

        let mut spec = ColorSpec::new();
        let (r, g, b) = (msg.color.r, msg.color.g, msg.color.b);

        spec.set_fg(Some(Color::Rgb(r, g, b)));
        buf.set_color(&spec).expect("set color");

        write!(buf, "{}", name).unwrap();
        buf.reset().expect("reset");

        if msg.is_action {
            write!(buf, " ").unwrap();
        } else {
            write!(buf, ": ").unwrap();
        }
    }

    fn write_lines(&self, buf: &mut termcolor::Buffer, lines: &[String]) {
        let pad: String = std::iter::repeat(" ")
            .take(self.opts.name_max + 2)
            .collect::<String>();

        for (i, s) in lines.iter().map(|s| s.trim_end()).enumerate() {
            if i == 0 {
                write!(buf, "{}", s).unwrap();
            } else {
                write!(buf, "{}{}{}", self.opts.left, pad, s).unwrap();
            }
            if lines.len() == 1 {
                writeln!(buf).unwrap();
                continue;
            }
            if i < lines.len() - 1 {
                let len = self.opts.line_max.saturating_sub(s.width());
                writeln!(buf, "{: >width$}", self.opts.right, width = len).unwrap();
            } else {
                writeln!(buf).unwrap();
            }
        }
    }

    fn split_line(&self, data: &str) -> Vec<String> {
        let max = self.opts.line_max;

        let mut lines = vec![];
        let mut line = String::new();
        let mut buf = String::new();

        for s in data.split_word_bounds() {
            if s.width() < max {
                if line.width() + s.width() >= max {
                    lines.push(line.clone());
                    line.clear();
                }
                line.push_str(&s);
                continue;
            }

            for c in s.chars() {
                if line.width() == max {
                    lines.push(line.clone());
                    line.clear();
                }
                if buf.width() == max {
                    line.push_str(&buf);
                    buf.clear();
                }
                buf.push(c);
            }

            if line.width() == max {
                lines.push(line.clone());
                line.clear();
            }
            if !buf.is_empty() {
                line.push_str(&buf)
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
        lines
    }
}

fn truncate(s: &mut String, max: usize) {
    if s.width() <= max {
        return;
    }
    let max = max.saturating_sub(1);
    s.truncate(max);
    s.insert(max, '…');
}
