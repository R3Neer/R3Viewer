# Markdown D2b1 R3: incorporación de biblioteca

2026-10-06. Se incorpora FlowHandle, ventanas Unicode y navegación Line/Resize con continuación visual independiente de RawAnchor. Base compartida 97f07e8a0afe0cbae1faebc858dfaebac953501a. R1/R2 y sus rechazos se conservan.

## Evidencia y composición

Candidato 624 registros/583 archivos: MANIFEST444363CFD182D478AB7C8965D7E048A90AB3B665F73C980898FE804160B9D571. Aceptación independiente: informe FD7A2B58FF98CBBDC36D4AB8070A04F05497879320FFAE4B61532476148AB5A3; manifiesto04B1E09D9509A1100B17A51EA55AE2BC7EBF77242A9709359E67B022DEB3A40D,39 evidencias. Raíz verificó todos los registros sin discrepancias antes de modificar.

Patch normalizado480D02B2009A941AD1D91BC5D449AD244B8483884B4F132D3B081D046BA2A6A4. El check inicial completo rechazó sólo lib.rs porque ya registra Code/cache/Fuente inline posteriores; ningún cambio se aplicó en ese intento. Se verificó y aplicó el delta de ocho fuentes excluyendo lib, y se agregaron únicamente las declaraciones document_layout/document_view, preservando todos los módulos posteriores. ChargedVec conserva same del presupuesto ya integrado. No se sustituyó el lib antiguo completo.

La combinación real ejecutó 68 pruebas:17 document_view,16 Flow,16 Code y19 preparation_owner. Todas PASS. El filtro source_preparation::tests seleccionó cero casos: log conservado, no cuenta como prueba; los19 del propietario sí ejercitan HASH/SOURCE y alternancia. Formato PASS; Clippy de todos los targets PASS, sin warnings (exit0). Evidencia privada markdown-d2b1-root-combination-r3, sin perfiles ni publicación.

## Alcance y pendientes

Los dos defectos críticos están resueltos por controles independientes originales: ancla inválida rechaza antes de Reference y C1 permite avanzar A\u{ → 0085 → }Z. LayoutAnchor conserva parte visual al redimensionar; RawAnchor sigue físico para Fuente y Code. Referencia, cuotas y rechazo conservan current/history.

No cierra RV-301 ni b2/b3/pump/CLI/gráficos/Unix/QA visual. RefID/roles y salida inline siguen en combinación privada separada. El diagnóstico de documento ASCII denso aún devuelve Reference por Work; se investiga hint y recorridos, sin adoptar cuotas nuevas ni atribuir usabilidad completa a68 pruebas.
