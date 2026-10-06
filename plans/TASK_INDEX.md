# All remediation task cards

Coordinator reference only. All 67 original topics are mapped once; priorities and task edges are preserved from the cutoff. Current progress belongs in [STATUS.md](STATUS.md). Do not give an Agent this entire table as its assignment.

| Task / issue | Package | Priority | Owner | Profile | Task predecessors |
| --- | --- | --- | --- | --- | --- |
| [R26-suite-selection](tasks/R26-suite-selection.md) / [#287](https://github.com/BerryUIKI/Omera/issues/287) | [W01](W01_PLAN.md) | P2 | QA engineer | V2 | None beyond baseline/ownership |
| [R26-batch-test](tasks/R26-batch-test.md) / [#288](https://github.com/BerryUIKI/Omera/issues/288) | [W01](W01_PLAN.md) | P2 | UI QA engineer | V2 | None beyond baseline/ownership |
| [R26-format](tasks/R26-format.md) / [#294](https://github.com/BerryUIKI/Omera/issues/294) | [W01](W01_PLAN.md) | P2 | Rust engineer | V0 | None beyond baseline/ownership |
| [R28-handoff](tasks/R28-handoff.md) / [#290](https://github.com/BerryUIKI/Omera/issues/290) | [W01](W01_PLAN.md) | P2 | Documentation engineer with lead sign-off | V5 | None beyond baseline/ownership |
| [R28-api](tasks/R28-api.md) / [#291](https://github.com/BerryUIKI/Omera/issues/291) | [W01](W01_PLAN.md) | P2 | Lead contract/documentation engineer | V5 | None beyond baseline/ownership |
| [R28-inventory](tasks/R28-inventory.md) / [#292](https://github.com/BerryUIKI/Omera/issues/292) | [W01](W01_PLAN.md) | P2 | Documentation/QA engineer | V5 | None beyond baseline/ownership |
| [R26](tasks/R26.md) / [#108](https://github.com/BerryUIKI/Omera/issues/108) | [W01](W01_PLAN.md) | P2 | QA engineer | V2 | [R26-suite-selection](tasks/R26-suite-selection.md), [R26-batch-test](tasks/R26-batch-test.md) |
| [R26-ipc-ci](tasks/R26-ipc-ci.md) / [#286](https://github.com/BerryUIKI/Omera/issues/286) | [W01](W01_PLAN.md) | P2 | QA engineer | V5 | [R28-inventory](tasks/R28-inventory.md) |
| [R11](tasks/R11.md) / [#241](https://github.com/BerryUIKI/Omera/issues/241) | [W02](W02_PLAN.md) | P1 | Lead storage engineer | V1 | [R26-format](tasks/R26-format.md) |
| [R04-source](tasks/R04-source.md) / [#269](https://github.com/BerryUIKI/Omera/issues/269) | [W03](W03_PLAN.md) | P1 | Lead migration engineer | V1 | None beyond baseline/ownership |
| [R01](tasks/R01.md) / [#98](https://github.com/BerryUIKI/Omera/issues/98) | [W03](W03_PLAN.md) | P1 | Lead recovery engineer | V3 | None beyond baseline/ownership |
| [R04](tasks/R04.md) / [#234](https://github.com/BerryUIKI/Omera/issues/234) | [W03](W03_PLAN.md) | P1 | Lead migration/configuration engineer | V3 | [R04-source](tasks/R04-source.md) |
| [R05](tasks/R05.md) / [#235](https://github.com/BerryUIKI/Omera/issues/235) | [W03](W03_PLAN.md) | P1 | Lead startup/migration engineer | V3 | [R04](tasks/R04.md) |
| [R04-cleanup](tasks/R04-cleanup.md) / [#270](https://github.com/BerryUIKI/Omera/issues/270) | [W03](W03_PLAN.md) | P1 | Lead cleanup/security engineer | V3 | [R04](tasks/R04.md), [R05](tasks/R05.md) |
| [R05-ui](tasks/R05-ui.md) / [#271](https://github.com/BerryUIKI/Omera/issues/271) | [W03](W03_PLAN.md) | P2 | Migration UI engineer; lead owns backend contracts | V3 | [R04](tasks/R04.md), [R05](tasks/R05.md), [R28-api](tasks/R28-api.md) |
| [R33](tasks/R33.md) / [#262](https://github.com/BerryUIKI/Omera/issues/262) | [W04](W04_PLAN.md) | P2 | Lead file-service engineer | V3 | [R11](tasks/R11.md) |
| [R33-pipeline](tasks/R33-pipeline.md) / [#293](https://github.com/BerryUIKI/Omera/issues/293) | [W04](W04_PLAN.md) | P2 | Lead file-service engineer | V3 | [R33](tasks/R33.md) |
| [R31](tasks/R31.md) / [#260](https://github.com/BerryUIKI/Omera/issues/260) | [W04](W04_PLAN.md) | P2 | Lead file-service engineer | V1 | [R33](tasks/R33.md) |
| [R08](tasks/R08.md) / [#238](https://github.com/BerryUIKI/Omera/issues/238) | [W04](W04_PLAN.md) | P1 | Lead transform/persistence engineer | V1 | [R33](tasks/R33.md) |
| [R32](tasks/R32.md) / [#261](https://github.com/BerryUIKI/Omera/issues/261) | [W04](W04_PLAN.md) | P2 | Lead file-operations engineer | V1 | [R33](tasks/R33.md) |
| [R02](tasks/R02.md) / [#232](https://github.com/BerryUIKI/Omera/issues/232) | [W04](W04_PLAN.md) | P1 | Lead pipeline engineer | V1 | [R33-pipeline](tasks/R33-pipeline.md) |
| [R02-dedup](tasks/R02-dedup.md) / [#268](https://github.com/BerryUIKI/Omera/issues/268) | [W04](W04_PLAN.md) | P1 | Lead import/pipeline engineer | V1 | [R33](tasks/R33.md), [R33-pipeline](tasks/R33-pipeline.md) |
| [R03](tasks/R03.md) / [#233](https://github.com/BerryUIKI/Omera/issues/233) | [W04](W04_PLAN.md) | P1 | Lead cleanup engineer | V1 | [R11](tasks/R11.md), [R02](tasks/R02.md) |
| [R06](tasks/R06.md) / [#236](https://github.com/BerryUIKI/Omera/issues/236) | [W05](W05_PLAN.md) | P1 | Metadata/privacy engineer; lead review | V1 | [R33](tasks/R33.md) |
| [R07](tasks/R07.md) / [#237](https://github.com/BerryUIKI/Omera/issues/237) | [W05](W05_PLAN.md) | P1 | Metadata/media engineer; lead review | V1 | [R33](tasks/R33.md) |
| [R36](tasks/R36.md) / [#296](https://github.com/BerryUIKI/Omera/issues/296) | [W05](W05_PLAN.md) | P2 | Metadata engineer; lead review for override persistence | V1 | None beyond baseline/ownership |
| [R07-export](tasks/R07-export.md) / [#272](https://github.com/BerryUIKI/Omera/issues/272) | [W05](W05_PLAN.md) | P1 | Metadata/export engineer; lead review | V1 | [R07](tasks/R07.md) |
| [R07-sidecars](tasks/R07-sidecars.md) / [#273](https://github.com/BerryUIKI/Omera/issues/273) | [W05](W05_PLAN.md) | P1 | Metadata/import engineer; lead review | V1 | [R06](tasks/R06.md), [R07](tasks/R07.md) |
| [R09-privacy](tasks/R09-privacy.md) / [#274](https://github.com/BerryUIKI/Omera/issues/274) | [W05](W05_PLAN.md) | P1 | Lead transform/privacy engineer | V1 | [R07](tasks/R07.md), [R08](tasks/R08.md) |
| [R09](tasks/R09.md) / [#239](https://github.com/BerryUIKI/Omera/issues/239) | [W05](W05_PLAN.md) | P2 | Media/storage engineer; lead review | V1 | [R07](tasks/R07.md), [R08](tasks/R08.md) |
| [R10](tasks/R10.md) / [#240](https://github.com/BerryUIKI/Omera/issues/240) | [W05](W05_PLAN.md) | P2 | Media engineer | V1 | [R07](tasks/R07.md) |
| [R23](tasks/R23.md) / [#254](https://github.com/BerryUIKI/Omera/issues/254) | [W05](W05_PLAN.md) | P1 | Lead import engineer | V3 | [R33](tasks/R33.md), [R07-sidecars](tasks/R07-sidecars.md) |
| [R10-discovery](tasks/R10-discovery.md) / [#267](https://github.com/BerryUIKI/Omera/issues/267) | [W05](W05_PLAN.md) | P2 | Scan/media engineer | V1 | [R10](tasks/R10.md) |
| [R12](tasks/R12.md) / [#243](https://github.com/BerryUIKI/Omera/issues/243) | [W06](W06_PLAN.md) | P1 | Lead scan/reconciliation engineer | V1 | None beyond baseline/ownership |
| [R16](tasks/R16.md) / [#247](https://github.com/BerryUIKI/Omera/issues/247) | [W06](W06_PLAN.md) | P2 | Pipeline/scan engineer; lead file-safety review | V3 | [R33-pipeline](tasks/R33-pipeline.md), [R02](tasks/R02.md), [R03](tasks/R03.md) |
| [R34](tasks/R34.md) / [#266](https://github.com/BerryUIKI/Omera/issues/266) | [W06](W06_PLAN.md) | P2 | Scan/UI engineer; lead review of scan lifecycle | V4 | [R12](tasks/R12.md) |
| [R13](tasks/R13.md) / [#244](https://github.com/BerryUIKI/Omera/issues/244) | [W07](W07_PLAN.md) | P1 | Lead sync/storage engineer | V3 | [R11](tasks/R11.md), [R33](tasks/R33.md) |
| [R14](tasks/R14.md) / [#245](https://github.com/BerryUIKI/Omera/issues/245) | [W07](W07_PLAN.md) | P1 | Sync engineer; lead contract review | V1 | None beyond baseline/ownership |
| [R25](tasks/R25.md) / [#202](https://github.com/BerryUIKI/Omera/issues/202) | [W07](W07_PLAN.md) | P2 | Lead transform/concurrency engineer | V3 | [R08](tasks/R08.md) |
| [R25-estimate](tasks/R25-estimate.md) / [#284](https://github.com/BerryUIKI/Omera/issues/284) | [W07](W07_PLAN.md) | P2 | Lead export/concurrency engineer | V3 | None beyond baseline/ownership |
| [R25-export](tasks/R25-export.md) / [#285](https://github.com/BerryUIKI/Omera/issues/285) | [W07](W07_PLAN.md) | P2 | Lead export/concurrency engineer | V3 | [R06](tasks/R06.md), [R07-export](tasks/R07-export.md) |
| [R14-webdav](tasks/R14-webdav.md) / [#275](https://github.com/BerryUIKI/Omera/issues/275) | [W07](W07_PLAN.md) | P1 | Sync engineer; lead contract review | V1 | [R14](tasks/R14.md) |
| [R14-s3](tasks/R14-s3.md) / [#276](https://github.com/BerryUIKI/Omera/issues/276) | [W07](W07_PLAN.md) | P1 | Sync engineer; lead contract review | V1 | [R14](tasks/R14.md) |
| [R24](tasks/R24.md) / [#255](https://github.com/BerryUIKI/Omera/issues/255) | [W07](W07_PLAN.md) | P2 | Lead sync/concurrency engineer | V1 | [R14](tasks/R14.md) |
| [R24-cancel](tasks/R24-cancel.md) / [#283](https://github.com/BerryUIKI/Omera/issues/283) | [W07](W07_PLAN.md) | P2 | Lead sync/concurrency engineer | V1 | [R24](tasks/R24.md) |
| [R17-mirror](tasks/R17-mirror.md) / [#277](https://github.com/BerryUIKI/Omera/issues/277) | [W08](W08_PLAN.md) | P2 | Configuration engineer; lead revision review | V3 | None beyond baseline/ownership |
| [R17-revision](tasks/R17-revision.md) / [#278](https://github.com/BerryUIKI/Omera/issues/278) | [W08](W08_PLAN.md) | P2 | Configuration/UI engineer; lead revision review | V3 | None beyond baseline/ownership |
| [R17](tasks/R17.md) / [#248](https://github.com/BerryUIKI/Omera/issues/248) | [W08](W08_PLAN.md) | P2 | Settings UI engineer | V2 | [R17-mirror](tasks/R17-mirror.md), [R17-revision](tasks/R17-revision.md) |
| [R18](tasks/R18.md) / [#249](https://github.com/BerryUIKI/Omera/issues/249) | [W09](W09_PLAN.md) | P2 | Frontend state engineer | V2 | None beyond baseline/ownership |
| [R18-concurrency](tasks/R18-concurrency.md) / [#279](https://github.com/BerryUIKI/Omera/issues/279) | [W09](W09_PLAN.md) | P2 | Frontend state engineer | V2 | [R18](tasks/R18.md) |
| [R19](tasks/R19.md) / [#250](https://github.com/BerryUIKI/Omera/issues/250) | [W09](W09_PLAN.md) | P2 | Frontend state engineer | V4 | [R18](tasks/R18.md), [R18-concurrency](tasks/R18-concurrency.md) |
| [R21](tasks/R21.md) / [#252](https://github.com/BerryUIKI/Omera/issues/252) | [W10](W10_PLAN.md) | P2 | Gallery engineer | V4 | None beyond baseline/ownership |
| [R30](tasks/R30.md) / [#259](https://github.com/BerryUIKI/Omera/issues/259) | [W10](W10_PLAN.md) | P2 | Gallery engineer | V4 | [R19](tasks/R19.md) |
| [R35](tasks/R35.md) / [#295](https://github.com/BerryUIKI/Omera/issues/295) | [W10](W10_PLAN.md) | P2 | Gallery engineer; lead owns query-selection contract | V4 | [R19](tasks/R19.md), [R30](tasks/R30.md) |
| [R29-table](tasks/R29-table.md) / [#263](https://github.com/BerryUIKI/Omera/issues/263) | [W10](W10_PLAN.md) | P2 | Gallery/UI engineer | V4 | [R30](tasks/R30.md) |
| [R37](tasks/R37.md) / [#297](https://github.com/BerryUIKI/Omera/issues/297) | [W10](W10_PLAN.md) | P2 | Gallery/UI engineer | V4 | [R21](tasks/R21.md), [R36](tasks/R36.md) |
| [R20](tasks/R20.md) / [#251](https://github.com/BerryUIKI/Omera/issues/251) | [W11](W11_PLAN.md) | P2 | Statistics engineer; lead review for SQL | V3 | None beyond baseline/ownership |
| [R22](tasks/R22.md) / [#253](https://github.com/BerryUIKI/Omera/issues/253) | [W11](W11_PLAN.md) | P2 | Accessibility/UI engineer | V2 | None beyond baseline/ownership |
| [R22-sidebar](tasks/R22-sidebar.md) / [#281](https://github.com/BerryUIKI/Omera/issues/281) | [W11](W11_PLAN.md) | P2 | Accessibility/UI engineer | V2 | None beyond baseline/ownership |
| [R22-labels](tasks/R22-labels.md) / [#282](https://github.com/BerryUIKI/Omera/issues/282) | [W11](W11_PLAN.md) | P2 | Accessibility/UI engineer | V4 | [R35](tasks/R35.md) |
| [R29](tasks/R29.md) / [#258](https://github.com/BerryUIKI/Omera/issues/258) | [W11](W11_PLAN.md) | P2 | Localization/UI engineer | V2 | [R17](tasks/R17.md), [R18](tasks/R18.md) |
| [R20-denominator](tasks/R20-denominator.md) / [#280](https://github.com/BerryUIKI/Omera/issues/280) | [W11](W11_PLAN.md) | P2 | Statistics engineer; lead review for SQL | V3 | [R20](tasks/R20.md) |
| [R29-tags](tasks/R29-tags.md) / [#264](https://github.com/BerryUIKI/Omera/issues/264) | [W11](W11_PLAN.md) | P2 | Sidebar/UI engineer | V2 | [R22-sidebar](tasks/R22-sidebar.md) |
| [R15](tasks/R15.md) / [#246](https://github.com/BerryUIKI/Omera/issues/246) | [W12](W12_PLAN.md) | P2 | Lead backup/scheduler engineer | V3 | [R01](tasks/R01.md), [R17](tasks/R17.md) |
| [R28](tasks/R28.md) / [#257](https://github.com/BerryUIKI/Omera/issues/257) | [W12](W12_PLAN.md) | P2 | Documentation engineer | V5 | None beyond baseline/ownership |
| [R28-remote-db](tasks/R28-remote-db.md) / [#289](https://github.com/BerryUIKI/Omera/issues/289) | [W12](W12_PLAN.md) | P2 | Documentation engineer | V5 | None beyond baseline/ownership |
| [R27](tasks/R27.md) / [#256](https://github.com/BerryUIKI/Omera/issues/256) | [W13](W13_PLAN.md) | P2 | Lead release engineer | V6 | [R01](tasks/R01.md), [R26](tasks/R26.md), [R26-ipc-ci](tasks/R26-ipc-ci.md) |
