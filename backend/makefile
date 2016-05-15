run: compile-acorn compile-c link-llvm optimize-llvm compile-llvm compile-assembly run-binary

compile-acorn:
	cargo run -q > build/acorn.ll

compile-c:
	clang lib/* -S -emit-llvm -o build/lib.ll

link-llvm:
	llvm-link build/acorn.ll build/lib.ll -o build/out.as

optimize-llvm:
	cat build/out.as | opt -mem2reg > build/out.lc

compile-llvm:
	llc build/out.lc -o build/out.s

compile-assembly:
	clang build/out.s -o build/a.out


run-binary:
	./build/a.out

clean:
	rm -f build/*
	find . | grep '\.bk$$' | xargs rm -f

test:
	cargo test
