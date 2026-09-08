OPENQASM 2.0;
include "qelib1.inc";

qreg q[3];
creg c[3];

// H on q[0], then some gates on q[1] that commute with H on q[0],
// then another H on q[0] — commutation should bring the two H gates
// together and cancel them.
h q[0];
rz(1.5708) q[1];
rz(0.7854) q[2];
h q[0];

// Rz sequence on q[1] that commutation + rotation merging should collapse
rz(0.3927) q[1];
rz(-1.9635) q[1];
rz(0.1963) q[1];

measure q[0] -> c[0];
measure q[1] -> c[1];
measure q[2] -> c[2];