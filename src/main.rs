use std::{
    collections::HashMap,
    io::{self, BufWriter, Write},
    time::Instant,
};

fn multiply(val: u16) -> (u8, u8, u8) {
    let calc = val.isqrt();

    let mut offset = val - calc * calc;

    let inner = if (offset % calc) != calc {
        let inner = calc + (offset / calc);
        offset %= calc;
        inner
    } else {
        calc
    };

    (offset as u8, calc as u8, inner as u8)
}

fn summations() -> HashMap<i16, String> {
    let mut out = HashMap::new();

    for val in -257..=257i16 {
        match val.signum() {
            sign @ (1 | -1) => {
                let abs = val.unsigned_abs();
                let code = if sign == 1 { "+" } else { "-" };

                let raw = format!(">{}>", code.repeat(usize::from(abs)));

                let (offset, outer, inner) = multiply(abs);

                let mult = format!(
                    "{}[>{}<-]>{}>",
                    "+".repeat(outer as usize),
                    code.repeat(inner as usize),
                    code.repeat(offset as usize)
                );

                if mult.len() > raw.len() {
                    out.insert(val, raw);
                } else {
                    out.insert(val, mult);
                }
            }
            0 => {
                out.insert(0, ">>".into());
            }
            _ => unreachable!(),
        }
    }

    out
}

fn occur_map(reader: &[u8]) -> io::Result<[u64; 256]> {
    let mut counts = [0; _];

    for b in reader {
        counts[usize::from(*b)] += 1;
    }

    Ok(counts)
}

fn best_offset(compiled: &HashMap<i16, String>, counts: [u64; 256]) -> u8 {
    let mut min_bytes = None::<(u8, u64)>;

    for t in 0..=255i16 {
        let mut bytes = 0;

        for (b, amount) in counts.iter().enumerate() {
            bytes += compiled
                .get(&(b as i16 - t))
                .expect("-257..=257 is set")
                .len() as u64
                * amount;
        }

        if min_bytes.is_none_or(|(_, size)| bytes < size) {
            min_bytes = Some((t as u8, bytes));
        }
    }

    min_bytes.unwrap().0
}

fn filesize(counts: [u64; 256]) -> u64 {
    counts.iter().sum()
}

fn to_bf<W: Write>(compiled: &HashMap<i16, String>, asc: &[u8], out: &mut W) -> io::Result<()> {
    let counts = occur_map(asc)?;

    let offset = best_offset(compiled, counts);

    let mut writer = BufWriter::new(out);

    let offset_s = &compiled[&(offset as i16)];

    writer.write_all(&offset_s.as_bytes()[..offset_s.len() - 1])?;
    writer.write_all(b"[-")?;
    for _ in 0..filesize(counts) {
        writer.write_all(b">>+")?;
    }
    for _ in 0..filesize(counts) {
        writer.write_all(b"<<")?;
    }
    writer.write_all(b"]>")?;

    let mut first = true;

    for &byte in asc.iter().rev() {
        let value = byte as i16 - offset as i16;

        writer.write_all(compiled[&value].as_bytes())?;

        if first {
            first = false;

            writer.write_all(b"<<->>")?;
        }
    }

    writer.write_all(b"+[<.<+]\n")?;

    writer.flush()?;

    Ok(())
}

fn main() -> io::Result<()> {
    let start = Instant::now();

    let data = std::fs::read(
        std::env::args()
            .nth(1)
            .ok_or(io::Error::other("missing filename"))?,
    )?;

    let compiled = summations();

    to_bf(&compiled, &data, &mut std::io::stdout().lock())?;

    eprintln!("completed in: {:?}", start.elapsed());

    Ok(())
}
