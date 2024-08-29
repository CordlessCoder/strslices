use std::time::Duration;

fn main() {
    let mut c = criterion::Criterion::default()
        .warm_up_time(Duration::from_millis(100))
        .measurement_time(Duration::from_millis(5000));

    for (name, input) in [
        ("ASCII-only", 'a'),
        ("2 byte chars", 'Ã'),
        ("3 byte chars", 'ค'),
        ("4 byte chars", '𒆣'),
    ] {
        let mut group = c.benchmark_group(name);
        group.bench_with_input(
            "16-byte LUT",
            input.to_string().repeat(1024).as_str(),
            |b, input| b.iter(|| input.char_slices().count()),
        );
        group.bench_with_input(
            "256-byte LUT",
            input.to_string().repeat(1024).as_str(),
            |b, input| b.iter(|| input.char_slices_large().count()),
        );
        group.finish()
    }
}

pub struct CharSlices<'s>(&'s str);

impl CharSlices<'_> {
    pub fn remaining(&self) -> &str {
        self.0
    }
}

impl<'s> Iterator for CharSlices<'s> {
    type Item = &'s str;

    fn next(&mut self) -> Option<&'s str> {
        const LUT: [u8; 16] = [
            1, 1, 1, 1, 1, 1, 1, 1, // 0x0*** => 1
            0, 0, 0, 0, // 0x10**, invalid(continuation byte)
            2, 2, // 0x110* => 2
            3, // 0x1110
            4, // 0x1111
        ];

        match self.0.len() {
            0 => None,
            1.. => {
                let lower = self.0.as_bytes()[0] >> 4;
                let end = LUT[lower as usize] as usize;
                let ch;
                (ch, self.0) = unsafe { self.0.split_at_checked(end).unwrap_unchecked() };
                Some(ch)
            }
        }
    }
}

pub trait CharSlicesStrExt {
    fn char_slices(&self) -> CharSlices<'_>;
}
impl CharSlicesStrExt for &str {
    fn char_slices(&self) -> CharSlices<'_> {
        CharSlices(self)
    }
}

pub struct CharSlicesLarge<'s>(&'s str);

impl CharSlicesLarge<'_> {
    pub fn remaining(&self) -> &str {
        self.0
    }
}

impl<'s> Iterator for CharSlicesLarge<'s> {
    type Item = &'s str;

    fn next(&mut self) -> Option<&'s str> {
        const LUT: [u8; 256] = {
            const LUT: [u8; 16] = [
                1, 1, 1, 1, 1, 1, 1, 1, // 0x0*** => 1
                0, 0, 0, 0, // 0x10**, invalid(continuation byte)
                2, 2, // 0x110* => 2
                3, // 0x1110
                4, // 0x1111
            ];
            let mut out: [u8; 256] = [0; 256];
            let mut i = 0;
            while i < 256 {
                out[i] = LUT[i / 16];
                i += 1;
            }
            out
        };

        match self.0.len() {
            0 => None,
            1.. => {
                let end = LUT[self.0.as_bytes()[0] as usize] as usize;
                let ch;
                (ch, self.0) = unsafe { self.0.split_at_checked(end).unwrap_unchecked() };
                Some(ch)
            }
        }
    }
}

pub trait CharSlicesLargeStrExt {
    fn char_slices_large(&self) -> CharSlicesLarge<'_>;
}
impl CharSlicesLargeStrExt for &str {
    fn char_slices_large(&self) -> CharSlicesLarge<'_> {
        CharSlicesLarge(self)
    }
}
