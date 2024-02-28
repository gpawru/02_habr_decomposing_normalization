# бенчмарки
bench:
	cd benches && cargo bench >> report.txt && cargo run report.txt && rm report.txt

# тесты
tests:
	cd tests && cargo test
