
use bevy_window::FrameState;
use pi_hash::XHashMap;
use pi_world::insert::Insert;
use pi_world::prelude::IntoSystemConfigs;
use pi_world::schedule_config::IntoSystemSetConfigs;
use pi_world::single_res::{SingleRes, SingleResMut};
use pi_world::system_params::SystemParam;
use pi_world::{prelude::Plugin, schedule::First, schedule_config::SystemSet, world::{Entity, World}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default)]
pub enum TraceOption {
    #[default]
    None,
    Record,
    Play,
}

// 运行状态
bitflags::bitflags! {
    #[repr(transparent)]
    #[derive(Default, Serialize, Deserialize)]
    pub struct RunState: u32 {
        const NONE          = 0;
        const SETTING       = (1 << 0); // 设置
        const LAYOUT        = (1 << 1); // 计算布局
        const MATRIX        = (1 << 2); // 计算世界矩阵
        // const RENDER     = (1 << 2); // 渲染
    }
}

pub const RECORD_UI_COMMAND: KeyRecord = 1;
pub const RECORD_D3_COMMAND: KeyRecord = 2;


#[derive(Debug, Clone, Hash, SystemSet, PartialEq, Eq)]
pub enum StageCMDTrace {
    Before,
    Trace,
    After,
}

/// 指令类型标识
pub type KeyRecord = u8;

/// 指令类型标识
#[derive(Default)]
pub enum EReplayResult {
    // 未进行播放
    #[default]
    None,
    // 已经播完， 不需要继续播放
    End,
    // 慢速播放设置
    AwaitFrame,
    // 还需要继续播放一些空帧
    NullFrame,
    // 已经播放到最后一个，设置当前播放状态
    IsLast,
    // 正常播放
    Ok,
}

/// 记录的单个类型指令数据集
pub type RecordItem = (KeyRecord, Vec<u8>);

/// 记录的一帧指令集
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Record {
    pub frame_index: usize, // 所在帧位置
    pub state: RunState,
    pub entities: Vec<Entity>,
    pub cmds: Vec<RecordItem>,
}

/// 指令的播放方法
pub type FnRecordPlay = fn(&mut World, &Vec<u8>, &XHashMap<Entity, Entity>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame {
    cur: usize,
    frames: usize,
}

// 播放状态
#[derive(Default)]
pub struct PlayState {
    pub next_state_index: usize, //
    // pub need_play_empty_count: usize,
    pub cur_frame_count: usize,

    // 节点对应表（由于某些原因， record中记录的Entity与实际的Entity不匹配）（比如录制的时候是在有spine渲染的情况下进行的，而播放得时候没有spine， 会造成gui创建的实体id不同）
    // 将指令描述中的Entity修改映射为当前创建的Entity
    pub node_map: XHashMap<Entity, Entity>,
    pub is_running: bool,

    pub next_reord_index: usize,

    // 播放速度
    pub speed: f32,

    // 等待帧数量（减速播放时需要用到）
    pub await_frame_count: usize,
    pub view_port: Option<Vec<f32>>,
    pub playresult: EReplayResult,
    pub option: TraceOption,
}

/// 全局指令记录器
#[derive(Default)]
pub struct Records {
    pub list: Vec<Record>,
    pub palycalls: XHashMap<KeyRecord, FnRecordPlay>,
    // 记录在每个状态下运行多少次
    pub run_state: Vec<(RunState, usize)>,
    pub cur_frame_count: usize,
}
impl Records {
    pub fn clear(&mut self) {
        self.list.clear();
    }
    pub fn frames(bin: &[u8]) -> Option<Vec<Record>> {
        match postcard::from_bytes::<Vec<Record>>(bin) {
            Ok(val) => Some(val),
            Err(_) => None,
        }
    }
    pub fn bin(&self) -> Vec<u8> {
		match postcard::to_stdvec::<Vec<Record>>(&self.list) {
			Ok(bin) => bin,
			Err(r) =>{
				log::error!("serialize fail!!, {:?}", r);
				Vec::<u8>::default()
			},
		}
    }
    pub fn record_create(&mut self, entity: Entity) {
        if let Some(list) = self.list.last_mut() {
            list.entities.push(entity);
        } else {
            log::error!(">>> list ilen is 0");
        }
    }
    pub fn record(&mut self, key: KeyRecord, serialized: Vec<u8>) {
        if let Some(list) = self.list.last_mut() {
            list.cmds.push((key, serialized));
        }
    }

    pub fn replay(world: &mut World) {
        log::error!(">>> replay");

        let records = world.get_single_res::<Records>().unwrap();
        let records_list_len = records.list.len();
        let play_state = world.get_single_res_mut::<PlayState>().unwrap();

        if !play_state.is_running {
            // // 清空指令列表
            // // 播放时， 忽略外部设置的任何其他指令， 只使用记录的指令来播放
            // let user_commands = world.get_single_res_mut::<UserCommands>().unwrap();
            // *user_commands = UserCommands::default();
            play_state.playresult = EReplayResult::End;
            return; // 已经播完， 不需要继续播放
        }

        // 已经播放到最后一个，设置当前播放状态
        if play_state.next_state_index >= records_list_len {
            play_state.is_running = false;
            play_state.cur_frame_count = 0;
            // **records = Records::default();
            play_state.playresult = EReplayResult::IsLast;
            return;
        }

        // // 慢速播放设置
        // if play_state.await_frame_count > 0 {
        //     play_state.await_frame_count -= 1;
        //     play_state.playresult = EReplayResult::AwaitFrame;
        //     return;
        // } else {
        //     if play_state.speed < 1.0 && play_state.speed > 0.0 {
        //         play_state.await_frame_count = (1.0 / play_state.speed) as usize;
        //     }
        // }

        play_state.cur_frame_count += 1;

        let cur_frame_count = play_state.cur_frame_count;
        let mut next_state_index = play_state.next_state_index;
    loop {
        let records = world.get_single_res::<Records>().unwrap();
        let r = &records.list[next_state_index];
        let frame_index = r.frame_index as f32 ;
        let play_state = world.get_single_res_mut::<PlayState>().unwrap();

        // 还需要继续播放一些空帧
        if frame_index as f32 > play_state.cur_frame_count  as f32 * play_state.speed {
            play_state.playresult = EReplayResult::NullFrame;
            break;
        }

        // if r.frame_index < cur_frame_count {
        //     let play_state = world.get_single_res_mut::<PlayState>().unwrap();
        //     play_state.playresult = EReplayResult::NullFrame;
        //     return;
        // }

        let records = world.get_single_res::<Records>().unwrap();
        let r = &records.list[next_state_index];
        // 创建Entity, 建立映射
        let node_map = {
            let createlist = r.entities.clone();
            let newlen = createlist.len();
            let mut newentities = vec![];
            for _ in 0..newlen {
                newentities.push(world.spawn_empty());
            }

            log::error!("Commands Count: {:?}", (r.cmds.len(), r.frame_index));
            log::error!("Entitiy Count: {:?}", r.entities);
            let play_state = world.get_single_res_mut::<PlayState>().unwrap();
            let node_map = &mut play_state.node_map;
            for i in 0..newlen {
                if node_map.contains_key(&createlist[i]) == false {
                    log::error!("{:?}", (createlist[i], newentities[i]));
                    node_map.insert(createlist[i], newentities[i]);
                }
            }

            node_map.clone()
        };

        let records = world.get_single_res::<Records>().unwrap();
        let r = &records.list[next_state_index];
        let palycalls = records.palycalls.clone();
        r.cmds.clone().iter().for_each(|item| {
            if let Some(playcall) = palycalls.get(&item.0) {
                playcall(world, &item.1, &node_map);
            }
        });

        let play_state = world.get_single_res_mut::<PlayState>().unwrap();
        play_state.next_state_index += 1;
        play_state.playresult = EReplayResult::Ok;
        next_state_index = play_state.next_state_index;
    }
    }
}

pub fn sys_cmd_record(
    mut records: SingleResMut<Records>,
    run_state: SingleRes<RunState>,
	frame_state: SingleRes<FrameState>,
) {
    records.cur_frame_count += 1;
    let cur_frame_count = records.cur_frame_count;
    if *run_state != RunState::SETTING  {
		if let FrameState::UnActive = *frame_state {
			records.run_state.push((*run_state, cur_frame_count));
		}
    }

    let frame_index = records.cur_frame_count;
    records.list.push(Record { frame_index, state: *run_state, ..Default::default() });
}

pub fn sys_cmd_replay(world: &mut World) {
    Records::replay(world);
}

pub fn sys_frame_count(
    mut records: SingleResMut<Records>,
) {
    records.cur_frame_count += 1; // 记录帧数量
}
pub struct GlobalCmdTracePlugin {
    pub option: TraceOption,
}
impl Plugin for GlobalCmdTracePlugin {
    fn build(&self, app: &mut pi_world::app::App) {

        app.configure_set(First, StageCMDTrace::Before         .before(StageCMDTrace::Trace));
        app.configure_set(First, StageCMDTrace::After         .after(StageCMDTrace::Trace));

        match self.option {
            TraceOption::Record => {
                app.add_system(First, sys_cmd_record
                    .in_set(StageCMDTrace::Trace)
                );
                app.add_system(First, sys_frame_count
                    .in_set(StageCMDTrace::After)
                );
            }
            TraceOption::Play => {
                app.add_system(First, sys_cmd_replay
                    .in_set(StageCMDTrace::Trace)
                );
            }
            TraceOption::None => {},
        };

        app.world.insert_single_res::<PlayState>(PlayState { option: self.option.clone(), ..Default::default() });
        app.world.init_single_res::<Records>();
        app.world.init_single_res::<RunState>();
    }
}