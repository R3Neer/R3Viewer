# Revisión del detalle JSON

2026-10-05. SIN_CRITICAS_PENDIENTES para detalle R3:17D2C7E144F7BC331460C3A06652E680321CD5289C6E29813133932137D28979. Informe independiente final3C153EE64C621DECBDA9B4F81DB544DD84582058F149A69A4551C3399C5D4E09. Base TOML integrado ccdb06d y corrección de identidad de recurso28f1212.

Tres precisiones cerradas sin reducir aceptación: JSON-P1, preflight local captura raw antes de cursor Jiter y decoder separado evita consumo doble; JSON-P2, detección de duplicados en padre usa heapsort fallible y comparación por bloques≤4KiB con cancel/deadline, no espera protección del Job del hijo; JSON-P3, igualdad de grupo solo compara texto decodificado, ordinal solo desempata al ordenar. Propuestas e informes intermedios temporales fueraGit. Adenda verifica working y blobTOML exactos SHA06B80C0D48F6BC9E9B7339E1D2C79ACD1791C61465DAB4A334BDAE1723268F93, sin nuevo cambio de gramática.

Diseño aprobado, no producto JSON implementado ni aceptación acreditada. [Detalle final](../sprints/S03-D2-core.md), contrato J01–J09 y gate del parser siguen vigentes. Implementación, candidato exacto, adversario y AE separados; visual/matriz pendientes. RV106 abierto.
