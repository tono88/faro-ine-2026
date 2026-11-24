#!/usr/bin/env python3
"""Exporta Places365 ResNet18 (MIT CSAIL) a un solo archivo ONNX.

Entrada : resnet18_places365.pth.tar (punto de control oficial: state_dict con prefijo "module.").
Salida  : places365_resnet18.onnx
          entrada  "imagen"   float32 [1, 3, 224, 224]  (RGB, normalizada; ver PROCEDENCIA.md)
          salida   "puntajes" float32 [1, 365]          (logits; softmax fuera del modelo)
          opset 12, forma fija, pesos dentro del archivo (sin datos externos).

Uso:
    venv/bin/python -I exportar.py --pesos descargas/github_pahul/resnet18_places365.pth.tar \
                                   --salida places365_resnet18.onnx
"""
import argparse
import hashlib
import sys

import numpy as np
import onnx
import onnxruntime as ort
import torch
import torchvision

# SHA-256 del archivo de pesos con el que se hizo la exportación (ver PROCEDENCIA.md).
SHA256_PESOS = "2f4759217d470da2b803f8f66cd4488a066406b555a5fb95ee9a4663f9f05588"
OPSET = 12


def sha256(ruta):
    h = hashlib.sha256()
    with open(ruta, "rb") as f:
        for bloque in iter(lambda: f.read(1 << 20), b""):
            h.update(bloque)
    return h.hexdigest()


def cargar_modelo(ruta_pesos):
    # weights_only=True: el archivo solo puede traer tensores y tipos básicos; no se ejecuta
    # código del pickle. El punto de control oficial solo pide OrderedDict, FloatStorage y
    # _rebuild_tensor.
    punto = torch.load(ruta_pesos, map_location="cpu", weights_only=True)
    estado = {
        (k[len("module."):] if k.startswith("module.") else k): v
        for k, v in punto["state_dict"].items()
    }
    modelo = torchvision.models.resnet18(num_classes=365)
    modelo.load_state_dict(estado, strict=True)
    modelo.eval()
    meta = {k: v for k, v in punto.items() if k != "state_dict"}
    return modelo, meta


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--pesos", required=True)
    ap.add_argument("--salida", default="places365_resnet18.onnx")
    ap.add_argument("--sin-verificar-hash", action="store_true")
    a = ap.parse_args()

    huella = sha256(a.pesos)
    print(f"pesos: {a.pesos}\n  sha256 = {huella}")
    if huella != SHA256_PESOS and not a.sin_verificar_hash:
        sys.exit("El SHA-256 de los pesos no es el esperado; no se exporta.")

    modelo, meta = cargar_modelo(a.pesos)
    print(f"  metadatos del punto de control: {meta}")
    print(f"  parámetros: {sum(p.numel() for p in modelo.parameters())}")

    torch.manual_seed(0)
    ejemplo = torch.randn(1, 3, 224, 224, dtype=torch.float32)
    torch.onnx.export(
        modelo,
        (ejemplo,),
        a.salida,
        input_names=["imagen"],
        output_names=["puntajes"],
        opset_version=OPSET,
        do_constant_folding=True,
        dynamic_axes=None,      # forma fija 1x3x224x224
        export_params=True,
        dynamo=False,           # exportador clásico (TorchScript): el nuevo no baja a opset 12
        external_data=False,
    )

    # --- comprobaciones sobre el archivo escrito ---
    m = onnx.load(a.salida, load_external_data=False)
    onnx.checker.check_model(a.salida, full_check=True)
    externos = [t.name for t in m.graph.initializer if t.data_location == onnx.TensorProto.EXTERNAL]
    ops = {}
    for n in m.graph.node:
        clave = (n.domain or "ai.onnx", n.op_type)
        ops[clave] = ops.get(clave, 0) + 1

    def forma(v):
        return [d.dim_value if d.HasField("dim_value") else d.dim_param for d in v.type.tensor_type.shape.dim]

    print(f"onnx: {a.salida}")
    print(f"  ir_version = {m.ir_version}; opsets = {[(o.domain or 'ai.onnx', o.version) for o in m.opset_import]}")
    print(f"  entradas = {[(v.name, forma(v)) for v in m.graph.input]}")
    print(f"  salidas  = {[(v.name, forma(v)) for v in m.graph.output]}")
    print(f"  operadores = {dict(sorted(ops.items()))}")
    print(f"  inicializadores con datos externos = {externos}")
    print("  onnx.checker (full_check) = correcto")
    assert not externos, "el modelo no debe llevar datos externos"
    assert all(d == "ai.onnx" for d, _ in ops), "solo operadores estándar"
    assert [(v.name, forma(v)) for v in m.graph.input] == [("imagen", [1, 3, 224, 224])]
    assert [(v.name, forma(v)) for v in m.graph.output] == [("puntajes", [1, 365])]

    # --- PyTorch contra ONNX Runtime ---
    sesion = ort.InferenceSession(a.salida, providers=["CPUExecutionProvider"])
    peor = 0.0
    gen = torch.Generator().manual_seed(2026)
    entradas = [ejemplo, torch.zeros(1, 3, 224, 224)]
    entradas += [torch.randn(1, 3, 224, 224, generator=gen) for _ in range(8)]
    coincide_top1 = 0
    for x in entradas:
        with torch.no_grad():
            y_pt = modelo(x).numpy()
        (y_ox,) = sesion.run(["puntajes"], {"imagen": x.numpy()})
        peor = max(peor, float(np.abs(y_pt - y_ox).max()))
        coincide_top1 += int(y_pt.argmax() == y_ox.argmax())
    print(f"  PyTorch vs ONNX Runtime en {len(entradas)} entradas: "
          f"diferencia absoluta máxima de logits = {peor:.3e}; mismo top-1 en {coincide_top1}/{len(entradas)}")
    print(f"  sha256(onnx) = {sha256(a.salida)}")
    print(f"versiones: torch {torch.__version__}, torchvision {torchvision.__version__}, "
          f"onnx {onnx.__version__}, onnxruntime {ort.__version__}, numpy {np.__version__}")


if __name__ == "__main__":
    main()
