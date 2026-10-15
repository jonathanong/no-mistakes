fn main() {
    for (name, size, align) in no_mistakes::codebase::dependencies::extract::receiver_layout_probe()
    {
        println!("{name}\t{size}\t{align}");
    }
}
