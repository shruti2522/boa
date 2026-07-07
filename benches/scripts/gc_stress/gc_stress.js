// gc_stress.js
// Stresses the GC with allocation-heavy patterns

function buildTree(depth) {
    if (depth === 0) {
        return { leaf: "data" };
    }
    return {
        left: buildTree(depth - 1),
        right: buildTree(depth - 1),
    };
}

function processTree(node) {
    let sum = 0;
    if (node.left) sum += processTree(node.left);
    if (node.right) sum += processTree(node.right);
    if (node.leaf) sum += 1;
    return sum;
}

function makeClosures() {
    let closures = [];
    for (let i = 0; i < 100; i++) {
        let x = i;
        closures.push(() => x * 2);
    }
    return closures;
}

function churnObjects() {
    let arr = [];
    for (let i = 0; i < 500; i++) {
        let obj = {};
        for (let j = 0; j < 10; j++) {
            obj[`prop_${j}`] = j * i;
        }
        arr.push(obj);
    }
    return arr;
}

function main() {
    for (let i = 0; i < 50; i++) {
        // Tree churn
        let tree = buildTree(10);
        let res = processTree(tree);
        // Let it get collected
        tree = null;

        // Closure churn
        let closures = makeClosures();
        for (let c of closures) {
            c();
        }
        closures = null;

        // Object property churn
        let objects = churnObjects();
        let val = 0;
        for (let obj of objects) {
            val += obj.prop_5;
        }
        objects = null;
    }
}
