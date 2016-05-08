run: compile-acorn compile-ssa compile-c link-binary run-binary

compile-acorn:
	cargo run -q > build/out.ssa

compile-ssa:
	qbe build/out.ssa -o build/out.s

compile-c:
	gcc lib/* -S -o build/lib.s

link-binary:
	gcc build/out.s build/lib.s -o build/a.out

run-binary:
	./build/a.out

test:
	cargo test
