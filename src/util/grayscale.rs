lazy_static! {
    static ref GSCALE: Vec<char> = {
        let tables = r#"$@B%8&WM#*oahkbdpqwmZO0QLCJUYXzcvunxrjft/\|()1{}[]?-_+~i!lI;:,"^` "#;
        let mut vec: Vec<char> = tables.chars().collect();
        vec.reverse();
        vec
    };
    static ref GSCALE_NUM: usize = GSCALE.len();
}

pub fn grayscale_to_char(gs: f64) -> char {
    GSCALE[(gs * (*GSCALE_NUM - 1) as f64) as usize]
}
