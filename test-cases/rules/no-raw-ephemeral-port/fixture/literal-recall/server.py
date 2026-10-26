# Signed integer zero is still port zero; these fixtures are never executed.
sock.bind(("127.0.0.1", 0))
sock.bind(("127.0.0.1", +0))
sock.bind(("127.0.0.1", -0))
sock.bind((host, + 0x0))
sock.bind((host, - 0_0))
sock.bind(("::1", -0b0, 0, 0))
sock.bind((host, +8080))
sock.bind((host, -1))
sock.bind((host, port))
sock.bind((host, -0.0))
sock.bind((host, +0x1))
sock.bind((host, 0 + offset))
# sock.bind((host, -0))
pass
sock.bind((host, +0)) # no-mistakes-disable-line no-raw-ephemeral-port
# no-mistakes-disable-next-line no-raw-ephemeral-port
sock.bind((host, -0))
