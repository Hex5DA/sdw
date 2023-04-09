define i64 @main() {
  ; allocating 'a'
  %1 = alloca i64
  ; 'addition' binop
  %2 = add i64 8, 4
  store i64 %2, ptr %1
  ; dereferencing 'a'
  %3 = load i64, ptr %1
  ret i64 %3
}

