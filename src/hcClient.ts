import { AdminWebsocket, AppWebsocket, CellId, HoloHashB64, InstalledAppInfo } from '@holochain/client';
import { Annotation, AnnotationOutput, HolochainOutput, Paper, PaperOutput, StateMachineInput, StepStateMachineInput } from './types/types';
import { HeaderHashB64, EntryHashB64 } from "@holochain-open-dev/core-types"
import { SensemakerOutput } from './types/sensemaker';

export class HcClient {
  adminWs: AdminWebsocket;
  appWs: AppWebsocket;
  cellId: CellId;

  constructor(adminWs, appWs, cellId) {
    this.adminWs = adminWs;
    this.appWs = appWs;
    this.cellId = cellId;
  }

  static async initialize(appPort, adminPort) {
      let appWs = await AppWebsocket.connect('ws://localhost:' + appPort.toString());
      let adminWs = await AdminWebsocket.connect('ws://localhost:' + adminPort.toString());

      let info: InstalledAppInfo = await appWs.appInfo({
        installed_app_id: 'test-app',
      });
      let cellId: CellId = info.cell_data[0].cell_id;
      return new HcClient(adminWs, appWs, cellId);
  }

  async callZome(fn_name: string, payload: any): Promise<any> {
    return await this.appWs.callZome({
      cap_secret: null,
      cell_id: this.cellId,
      zome_name: 'paperz_main_zome',
      fn_name,
      payload,
      provenance: this.cellId[1],
    })
  }

  async set_sensemaker_cell_id(cellId: CellId): Promise<void> {
    await this.callZome('set_sensemaker_cell_id', cellId);
  }

  async get_sensemaker_cell_id(): Promise<CellId> {
    return (await this.callZome('get_sensemaker_cell_id', null)) as CellId;
  }

  /// Plain holochain widget calls
  async get_all_paperz(): Promise<Array<PaperOutput>> {
    return (await this.callZome('get_all_paperz', null)) as Array<PaperOutput>;
  }

  async get_annotations_for_paper(paper_entry_hash: EntryHashB64): Promise<Array<AnnotationOutput>> {
    return (await this.callZome('get_annotations_for_paper', paper_entry_hash)) as Array<AnnotationOutput>;
  }

  async upload_paper(payload): Promise<HeaderHashB64> {
    return (await this.callZome('upload_paper', payload)) as HeaderHashB64;
  }

  // Holochain call with sensemaker bridge call
  async create_annotation(annotation: Annotation): Promise<HolochainOutput> {
    return (await this.callZome('create_annotation', annotation)) as HolochainOutput;
  }

  // Sensemaker bridge calls
  async get_state_machine_init(path: string): Promise<SensemakerOutput | null> {
    return (await this.callZome('get_state_machine_init', path)) as SensemakerOutput | null;
  };

  async get_state_machine_comp(path: string): Promise<SensemakerOutput | null> {
    return (await this.callZome('get_state_machine_comp', path)) as SensemakerOutput | null;
  }

  async get_state_machine_data(target_entry_hash: EntryHashB64): Promise<SensemakerOutput | null> {
    return (await this.callZome('get_state_machine_data', target_entry_hash)) as SensemakerOutput | null;
  }

  async set_state_machine_comp(path: string, expr: string): Promise<boolean> {
    return (await this.callZome('set_state_machine_comp', {path, expr} as StateMachineInput)) as boolean;
  }

  async set_state_machine_init(path: string, expr: string): Promise<boolean> {
    return (await this.callZome('set_state_machine_init', {path, expr} as StateMachineInput)) as boolean;
  }

  async step_sm(path: string, entry_hash: EntryHashB64, action: string): Promise<void> {
    return await this.callZome('step_sm_remote', {path, entry_hash, action} as StepStateMachineInput);
  }
}
