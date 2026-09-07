process.env.FIREBASE_HOSTING = "1";
process.env.NEXT_PUBLIC_API_URL =
  process.env.NEXT_PUBLIC_API_URL || "https://memecoin-os.web.app";
require("child_process").execSync("npx next build", {
  stdio: "inherit",
  env: process.env,
  cwd: require("path").join(__dirname, ".."),
});
