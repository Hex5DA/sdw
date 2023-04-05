define i64 @main() {
  ; allocating 'bobirty'
  %bobirty.0 = alloca i64
  store i64 3, ptr %bobirty.0
  ; allocating 'a'
  %a.0 = alloca i64
  ; dereferencing 'bobirty'
  %bobirty.1 = load i64, ptr %bobirty.0
  %_at.0 = add i64 4, %bobirty.1
  store i64 %_at.0, ptr %a.0
  ; dereferencing 'a'
  %a.1 = load i64, ptr %a.0
  ret i64 %a.1
}

