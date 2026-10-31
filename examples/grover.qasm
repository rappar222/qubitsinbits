// Grover's 2-Qubit Search Algorithm for target |11>
OPENQASM 2.0;
include "qelib1.inc";

qreg q[2];
creg c[2];

// 1. Uniform superposition
h q[0];
h q[1];

// 2. Oracle (flips phase of |11>)
cz q[0], q[1];

// 3. Diffuser (Inversion about mean)
h q[0];
h q[1];
x q[0];
x q[1];
cz q[0], q[1];
x q[0];
x q[1];
h q[0];
h q[1];

// 4. Measure
measure q[0] -> c[0];
measure q[1] -> c[1];
