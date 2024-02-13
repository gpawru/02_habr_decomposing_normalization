# бенчмарки
bench:
	cd benches && cargo bench >> report.txt && cargo run report.txt && rm report.txt

# тесты
test:
	cd tests && cargo test

# очистка
clean:
	cd benches && cargo clean && rm Cargo.lock
	cd prepare && cargo clean && rm Cargo.lock
	cd source && cargo clean && rm Cargo.lock
	cd tests && cargo clean && rm Cargo.lock
	cd decomposing/1_base && cargo clean && rm Cargo.lock
	cd decomposing/2_opt && cargo clean && rm Cargo.lock

# подготовка данных
bake:
	cd prepare && cargo run
