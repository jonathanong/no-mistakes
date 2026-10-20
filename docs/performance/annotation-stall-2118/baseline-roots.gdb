set pagination off
set confirm off
handle SIGINT stop nopass
python
import gdb, threading, time, os, signal, json
output=open('/tmp/nm-annotation-stall-evidence/gdb-root-events.jsonl','w', buffering=1)
class Done(gdb.FinishBreakpoint):
 def __init__(self, path):
  super().__init__(gdb.newest_frame(), internal=True)
  self.path=path
 def stop(self):
  output.write(json.dumps({'event':'finished','path':self.path,'time':time.monotonic()})+'\n')
  return False
class Root(gdb.Breakpoint):
 def stop(self):
  length=int(gdb.parse_and_eval('$rdx'))
  pointer=int(gdb.parse_and_eval('$rsi'))
  path=bytes(gdb.selected_inferior().read_memory(pointer,length)).decode('utf8',errors='replace') if 0<length<10000 else 'invalid-path-registers'
  output.write(json.dumps({'event':'started','path':path,'time':time.monotonic()})+'\n')
  Done(path)
  return False
Root("'<no_mistakes::codebase::postgres::query_annotation::evaluate::Evaluator<&no_mistakes::codebase::check_facts::relative_scope::project_relative_executor_scopes::{closure#4}::{closure#0}>>::run'", internal=True)
def interrupt_probe():
 time.sleep(60)
 os.kill(os.getpid(),signal.SIGINT)
threading.Thread(target=interrupt_probe,daemon=True).start()
end
run
thread apply all bt 40
kill
quit
