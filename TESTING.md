1. Speed test for the entire framework
`cargo test --release --test render_benchmark -- --nocapture`
2. Unit test for individual modules
`cargo test --lib camera` replace camera with the name of any module like CSG / Group etc
