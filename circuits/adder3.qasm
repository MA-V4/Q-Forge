OPENQASM 2.0;
include "qelib1.inc";

qreg q[7];
creg c[4];

x q[0];
x q[3];

ccx q[0],q[3],q[6];
cx q[0],q[3];
ccx q[1],q[3],q[5];
cx q[1],q[3];
cx q[1],q[5];
ccx q[2],q[3],q[4];
cx q[2],q[3];

measure q[3] -> c[0];
measure q[4] -> c[1];
measure q[5] -> c[2];
measure q[6] -> c[3];