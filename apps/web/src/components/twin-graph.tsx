"use client";

import { useMemo, useState } from "react";

export type GraphNode = {
  id: string;
  kind: string;
  label: string;
  importance: number;
  x: number;
  y: number;
  z?: number;
};

export type GraphEdge = {
  source: string;
  target: string;
  relationship_type: string;
};

export function TwinGraphView({
  entities,
  relationships,
  mode,
  highlightKind,
}: {
  entities: GraphNode[];
  relationships: GraphEdge[];
  mode: "2d" | "3d";
  highlightKind?: string;
}) {
  const [yaw, setYaw] = useState(0.6);
  const projected = useMemo(() => {
    return entities.map((n) => {
      const z = n.z ?? 0;
      if (mode === "2d") return { ...n, px: n.x, py: n.y };
      const c = Math.cos(yaw);
      const s = Math.sin(yaw);
      const x1 = n.x * c - z * s;
      const z1 = n.x * s + z * c;
      return { ...n, px: x1, py: n.y * 0.82 - z1 * 0.45 };
    });
  }, [entities, mode, yaw]);

  const xs = projected.map((n) => n.px);
  const ys = projected.map((n) => n.py);
  const minX = Math.min(0, ...xs) - 50;
  const minY = Math.min(0, ...ys) - 50;
  const maxX = Math.max(0, ...xs) + 80;
  const maxY = Math.max(0, ...ys) + 50;
  const byId = Object.fromEntries(projected.map((n) => [n.id, n]));

  if (entities.length === 0) {
    return <p className="text-sm text-mute">No Twin entities in this graph. Not an empty ecosystem — the projection has no nodes.</p>;
  }

  return (
    <div>
      {mode === "3d" ? (
        <input
          type="range"
          min={0}
          max={6.28}
          step={0.02}
          value={yaw}
          onChange={(e) => setYaw(Number(e.target.value))}
          className="w-full mb-2"
          aria-label="Rotate Twin"
        />
      ) : null}
      <svg viewBox={`${minX} ${minY} ${maxX - minX} ${maxY - minY}`} className="w-full h-80">
        {relationships.map((e, i) => {
          const a = byId[e.source];
          const b = byId[e.target];
          if (!a || !b) return null;
          return (
            <line
              key={i}
              x1={a.px}
              y1={a.py}
              x2={b.px}
              y2={b.py}
              stroke="#3ee58a55"
              strokeWidth={1}
            />
          );
        })}
        {projected.map((n) => {
          const hot = highlightKind && n.kind.toLowerCase().includes(highlightKind.toLowerCase());
          return (
            <g key={n.id}>
              <circle
                cx={n.px}
                cy={n.py}
                r={6 + n.importance * 6}
                fill={hot ? "#f0c14b" : "#3ee58a"}
                opacity={0.9}
              />
              <text x={n.px + 10} y={n.py + 4} fill="#8b95a8" fontSize="10" fontFamily="ui-monospace">
                {n.kind}:{n.label}
              </text>
            </g>
          );
        })}
      </svg>
      {mode === "3d" ? (
        <p className="text-xs text-mute mt-1">Isometric projection of the same Twin graph. Drag the slider to rotate. No decorative motion.</p>
      ) : null}
    </div>
  );
}
