OPENQASM 2.0;
include "qelib1.inc";

qreg q[3];
creg c[2];

// Prepare state to teleport on q[0]
x q[0];

// Create Bell pair on q[1], q[2]
h q[1];
cx q[1],q[2];

// Bell measurement
cx q[0],q[1];
h q[0];
measure q[0] -> c[0];
measure q[1] -> c[1];