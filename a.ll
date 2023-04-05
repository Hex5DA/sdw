define i1 @main() {
  ; allocating 'a'
  %a.0 = alloca i1
  store i1 0, ptr %a.0
  ; dereferencing 'a'
  %a.1 = load i1, ptr %a.0
  ret i1 %a.1
}

