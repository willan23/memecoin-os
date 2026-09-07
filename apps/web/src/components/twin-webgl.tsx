"use client";

import { useEffect, useRef } from "react";
import type { GraphEdge, GraphNode } from "./twin-graph";

const VERT = `
attribute vec3 aPos;
attribute vec3 aCol;
uniform mat4 uMVP;
varying vec3 vCol;
void main() {
  gl_Position = uMVP * vec4(aPos, 1.0);
  gl_PointSize = aPos.z * 0.0 + 8.0;
  vCol = aCol;
}
`;

const VERT_POINTS = `
attribute vec3 aPos;
attribute vec3 aCol;
attribute float aSize;
uniform mat4 uMVP;
varying vec3 vCol;
void main() {
  gl_Position = uMVP * vec4(aPos, 1.0);
  gl_PointSize = aSize;
  vCol = aCol;
}
`;

const FRAG = `
precision mediump float;
varying vec3 vCol;
void main() {
  gl_FragColor = vec4(vCol, 1.0);
}
`;

function compile(gl: WebGLRenderingContext, type: number, src: string) {
  const s = gl.createShader(type)!;
  gl.shaderSource(s, src);
  gl.compileShader(s);
  return s;
}

function program(gl: WebGLRenderingContext, vs: string) {
  const p = gl.createProgram()!;
  gl.attachShader(p, compile(gl, gl.VERTEX_SHADER, vs));
  gl.attachShader(p, compile(gl, gl.FRAGMENT_SHADER, FRAG));
  gl.linkProgram(p);
  return p;
}

function mul(a: number[], b: number[]) {
  const o = new Array(16).fill(0);
  for (let i = 0; i < 4; i++) {
    for (let j = 0; j < 4; j++) {
      o[i * 4 + j] =
        a[i * 4 + 0] * b[0 * 4 + j] +
        a[i * 4 + 1] * b[1 * 4 + j] +
        a[i * 4 + 2] * b[2 * 4 + j] +
        a[i * 4 + 3] * b[3 * 4 + j];
    }
  }
  return o;
}

function perspective(fovy: number, aspect: number, near: number, far: number) {
  const f = 1 / Math.tan(fovy / 2);
  const nf = 1 / (near - far);
  return [f / aspect, 0, 0, 0, 0, f, 0, 0, 0, 0, (far + near) * nf, -1, 0, 0, 2 * far * near * nf, 0];
}

function lookAt() {
  return [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, -420, 1];
}

function rotY(a: number) {
  const c = Math.cos(a);
  const s = Math.sin(a);
  return [c, 0, s, 0, 0, 1, 0, 0, -s, 0, c, 0, 0, 0, 0, 1];
}

function kindColor(kind: string, hot: boolean): [number, number, number] {
  if (hot) return [0.94, 0.76, 0.29];
  const k = kind.toLowerCase();
  if (k.includes("whale")) return [0.95, 0.45, 0.35];
  if (k.includes("pool") || k.includes("liq")) return [0.35, 0.75, 0.95];
  if (k.includes("nar")) return [0.7, 0.55, 0.95];
  return [0.24, 0.9, 0.54];
}

/** WebGL projection of Twin graph JSON. No idle motion. Pulse only when highlightKind is set. */
export function TwinWebGl({
  entities,
  relationships,
  highlightKind,
  yaw,
}: {
  entities: GraphNode[];
  relationships: GraphEdge[];
  highlightKind?: string;
  yaw: number;
}) {
  const ref = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = ref.current;
    if (!canvas) return;
    const gl = canvas.getContext("webgl");
    if (!gl) return;
    const lineProg = program(gl, VERT);
    const pointProg = program(gl, VERT_POINTS);
    const byId = Object.fromEntries(entities.map((n) => [n.id, n]));
    const lines: number[] = [];
    for (const e of relationships) {
      const a = byId[e.source];
      const b = byId[e.target];
      if (!a || !b) continue;
      lines.push(a.x, a.y, a.z ?? 0, 0.2, 0.35, 0.28, b.x, b.y, b.z ?? 0, 0.2, 0.35, 0.28);
    }
    const pts: number[] = [];
    for (const n of entities) {
      const hot = !!(highlightKind && n.kind.toLowerCase().includes(highlightKind.toLowerCase()));
      const [r, g, b] = kindColor(n.kind, hot);
      const size = 7 + n.importance * 10 + (hot ? 6 : 0);
      pts.push(n.x, n.y, n.z ?? 0, r, g, b, size);
    }
    const mvp = mul(perspective(0.9, canvas.width / canvas.height, 10, 2000), mul(lookAt(), rotY(yaw)));

    gl.viewport(0, 0, canvas.width, canvas.height);
    gl.clearColor(0.03, 0.035, 0.05, 1);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
    gl.enable(gl.DEPTH_TEST);

    if (lines.length) {
      gl.useProgram(lineProg);
      gl.uniformMatrix4fv(gl.getUniformLocation(lineProg, "uMVP"), false, new Float32Array(mvp));
      const buf = gl.createBuffer();
      gl.bindBuffer(gl.ARRAY_BUFFER, buf);
      gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(lines), gl.STATIC_DRAW);
      const pos = gl.getAttribLocation(lineProg, "aPos");
      const col = gl.getAttribLocation(lineProg, "aCol");
      gl.enableVertexAttribArray(pos);
      gl.enableVertexAttribArray(col);
      gl.vertexAttribPointer(pos, 3, gl.FLOAT, false, 24, 0);
      gl.vertexAttribPointer(col, 3, gl.FLOAT, false, 24, 12);
      gl.drawArrays(gl.LINES, 0, lines.length / 6);
    }

    if (pts.length) {
      gl.useProgram(pointProg);
      gl.uniformMatrix4fv(gl.getUniformLocation(pointProg, "uMVP"), false, new Float32Array(mvp));
      const buf = gl.createBuffer();
      gl.bindBuffer(gl.ARRAY_BUFFER, buf);
      gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(pts), gl.STATIC_DRAW);
      const pos = gl.getAttribLocation(pointProg, "aPos");
      const col = gl.getAttribLocation(pointProg, "aCol");
      const sz = gl.getAttribLocation(pointProg, "aSize");
      gl.enableVertexAttribArray(pos);
      gl.enableVertexAttribArray(col);
      gl.enableVertexAttribArray(sz);
      gl.vertexAttribPointer(pos, 3, gl.FLOAT, false, 28, 0);
      gl.vertexAttribPointer(col, 3, gl.FLOAT, false, 28, 12);
      gl.vertexAttribPointer(sz, 1, gl.FLOAT, false, 28, 24);
      gl.drawArrays(gl.POINTS, 0, pts.length / 7);
    }
  }, [entities, relationships, highlightKind, yaw]);

  return <canvas ref={ref} width={900} height={420} className="w-full h-80 bg-[#070910]" />;
}
