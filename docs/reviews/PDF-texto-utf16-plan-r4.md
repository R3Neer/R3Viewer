# PDF UTF16/mapping — revisión final del contrato R4

Adversario root, distinto del autor unix_r3_review. Candidato efectivo: R3 SHA256 0CDBA3171F996584B6D1D753856E20FC1B3F77DB98319E03AF3A8AFE833A4BEA más delta R4 SHA256 B3C74B5B2D41245FCDA87079967548EC082283967AFA8FEC9FECF9DAB1EB29A3. Las propuestas previas y la revisión R3 se conservan fuera de Git; los textos finales canónicos mantienen esos bytes y sus estados históricos de entrega.

Veredicto: **SIN_CRITICAS_PENDIENTES para el contrato operacional revisado**. Autoriza construir el gate privado, no demuestra runtime, ABI, memoria efectiva, cobertura semántica del PDF ni producto integrado.

- U16-P01: Native64/Unit16/Scalar40 y wire explícito. Cálculo independiente confirma M=8060928, output=8070600 <8388608, input=1049384, H=29757440 <40MiB y D=18677762 <24MiB. Receipt aritmético independiente conservado; no es prueba del allocator ni memoria interna PDFium.
- U16-P02: asociaciones por unidades en orden, incluidas no adyacentes/invertidas; no intervalos que incorporen índices excluidos. Errores API/mapping/cajas y diagnósticos Unicode están tipados y separados del texto legible.
- U16-P03: delta R4 define input válido como transporte completo, exige preflight antes de spawn y rechaza Failure temprano o errores finales del writer como HostError. No drenaje ilimitado, ACK ficticio ni validez por exit0. Controles adicionales congelados antes de outputs.

Se conserva una llamada GetText por página, decode cancelable en Rust, semántica SourceUnverified, originales nativos y FAIL históricos. La matriz original de18casos y suplementos R3/R4 requieren ejecución independiente sobre candidato congelado. La implementación, revisión de código, aceptación, integración, visual y ejecución Unix permanecen pendientes; RV-204/G1 no se cierran.
