define i64 @main() {
    %val = alloca i64
    store i64 5, ptr %val
    
    %valderef = load i64, ptr %val
    ret i64 %valderef
}
    
