impl Engine<'_> {
    fn await_result(&mut self, args: &[Expr], state: State) -> Option<Vec<(State, Value)>> {
        let [argument] = args else {
            return None;
        };
        let mut out = Vec::new();
        for (state, value) in self.expression(argument, state)? {
            let value = match value {
                Value::Promise(value) => *value,
                Value::Text(_) | Value::Bool(_) | Value::Null | Value::Undefined => value,
                _ => {
                    self.failures += 1;
                    continue;
                }
            };
            out.push((state, value));
        }
        Some(out)
    }

    fn execute_sql(
        &mut self,
        args: &[Expr],
        state: State,
        site: u32,
    ) -> Option<Vec<(State, Value)>> {
        let mut out = Vec::new();
        for (state, args) in self.all(&args.iter().collect::<Vec<_>>(), state)? {
            match args.first().and_then(|arg| state.text(arg)) {
                Some(text) => self.executed.push(Execution {
                    file: self.file()?.to_path_buf(),
                    start: site,
                    sql: text,
                }),
                None => self.failures += 1,
            }
            out.push((state, Value::Promise(Box::new(Value::Scalar))));
        }
        Some(out)
    }

    fn invoke_helper(
        &mut self,
        id: usize,
        args: &[Expr],
        state: State,
    ) -> Option<Vec<(State, Value)>> {
        let key = self.catalog.callee(self.file()?, id)?;
        let mut out = Vec::new();
        for (state, args) in self.all(&args.iter().collect::<Vec<_>>(), state)? {
            // Returned SQL may alias a read-only argument, including nested SQL.
            // Only calls without incoming SQL can establish a fresh builder.
            let owned_arguments = args.iter().all(Value::owned_argument);
            let Some(values) = self.run(&key, &args) else {
                self.failures += 1;
                continue;
            };
            for value in values {
                let Some(mut state) = self.copy_state(&state) else {
                    self.failures += 1;
                    continue;
                };
                let value = match value {
                    Value::Sql(text) if owned_arguments => state.allocate(text),
                    other => other,
                };
                out.push((state, value));
            }
        }
        Some(out)
    }
}
