# FlowHandle: contraste previo del delta D2b1

Sin críticas pendientes en esta propuesta de alias inmutable. Nota AC5577ACDE426E1F3298D52CF8D1CD9BFD4EE89B5D225585C34B0619DBAC8EBB leída contra FlowIndex y Navigator actuales. Autor unix_r3_review, revisor raíz independiente. No código ni runtime nuevo.

SharedFlow conserva un único FlowIndex en FlowAllocation. Navigator mantiene getter prestado y current/history; share_flow entrega sólo FlowHandle falible, noClone, de campos privados y sin mutación del índice. La closure recibe ese alias, no Navigator ni lifetime extendida. FlowLocation/cursores conservan sus contratos de identidad y cargos; compartir no permite copiar checkpoints ni cursors gratis.

El coste nuevo del Arc se reserva aparte de la identidad que FlowIndex ya cobra. El cuerpo de FlowIndex no se duplica; cada alias nuevo cobra su propio layout antes de clonar el Arc. El descriptor evita dobleconteo explícito, y un movimiento del mismo handle no es una nueva allocation. El código debe usar tamaños reales y checked arithmetic, distinguiendo layout solicitado de capacities de Vec y de RSS, conforme a la nota. Misma ConsumerBudget, old+new y cargos4096 vigentes.

Último alias conserva Flow/Document/Consumer/Actor hasta Drop físico aunque Navigator desaparezca. Resize no reconstruye Flow ni copia historia. Fold aún exige la transacción aprobada de remap Block/Run/raw y cotejo completo antes de commit; esta nota no aprueba replace_index ni ordinales de un Flow antiguo en otro nuevo.

La implementación b1 requiere b0 aceptado, nueva base con D4a integrado, revisión del delta y aceptación propia. Preoutputs de alias, cuota, current/history, cursor y remap siguen obligatorios antes de outputs. No prueba b1/layout, pump, ConPTY, backend, inline, Unix ni cierre RV. No añade un plan general o una dependencia ficticia.
