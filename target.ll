define i64 @main() {
    %x = alloca i64
    %xadd = add i64 3, 2
    store i64 %xadd, ptr %x

    %xderef = load i64, ptr %x
    ret i64 %xderef
}

