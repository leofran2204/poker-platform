import { Config } from "@remotion/cli/config";

// Quadros em JPEG de alta qualidade: mais rápido que PNG e sem perda visível
// depois da compressão H.264.
Config.setVideoImageFormat("jpeg");
Config.setJpegQuality(92);
Config.setOverwriteOutput(true);
