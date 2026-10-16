from phase_common import *

def restore_file_modes(dest,files,modes):
    # ZIP transport strips executable bits; authenticate every file before restoring any mode.
    require(set(files)==set(modes),'closed mode inventory mismatch')
    for name,digest in files.items():
        relative=Path(name);path=dest/relative
        require(not relative.is_absolute()and '..'not in relative.parts and path.resolve().is_relative_to(dest),
                'closed path escapes owned artifact')
        require(path.is_file()and not path.is_symlink()and sha(path)==digest,'closed artifact changed')
        require(type(modes[name])is int and 0<=modes[name]<=0o777,'unsafe closed file mode')
    for name,mode in modes.items():os.chmod(dest/name,mode,follow_symlinks=False)
