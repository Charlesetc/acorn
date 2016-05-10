run: compile-acorn compile-c link-llvm compile-llvm compile-assembly run-binary

compile-acorn:
	cargo run -q > build/out.ll

compile-c:
	clang lib/* -S -emit-llvm -o build/lib.ll

link-llvm:
	llvm-link build/out.ll build/lib.ll -o build/out.ll

compile-llvm:
	llc build/out.ll -o build/out.s

compile-assembly:
	clang build/out.s -o build/a.out


run-binary:
	./build/a.out

clean:
	rm -f build/*
	find . | grep '\.bk$$' | xargs rm -f

test:
	cargo test
