//! Code used from sushiibot
//! https://raw.githubusercontent.com/sushiibot/sushii-2/888fbcdaecc0838e5c3735a5aac677a2d327ef10/src/model/metrics.rs

use chrono::{NaiveDateTime, Utc};
use once_cell::sync::OnceCell;
use prometheus::{
    Gauge, GaugeVec, IntCounter, IntCounterVec, IntGauge, IntGaugeVec, Opts, Registry,
};
use prometheus_static_metric::make_static_metric;
use std::sync::Arc;

make_static_metric! {
    pub label_enum CpuUsageType {
        user,
        nice,
        system,
        interrupt,
        idle,
        iowait,
    }

    pub label_enum MemoryUsageType {
        available,
        free,
        total,
    }

    pub label_enum BlockStats {
        read_ios,
        read_merges,
        read_sectors,
        read_ticks,
        write_ios,
        write_merges,
        write_sectors,
        write_ticks,
        in_flight,
        io_ticks,
        time_in_queue,
    }

    pub label_enum SocketStats {
        tcp_sockets_in_use,
        tcp_sockets_orphaned,
        udp_sockets_in_use,
        tcp6_sockets_in_use,
        udp6_sockets_in_use,
    }

    pub label_enum NetworkStats {
        rx_bytes,
        tx_bytes,
        rx_packets,
        tx_packets,
        rx_errors,
        tx_errors,
    }

    pub label_enum LoadAvgStats {
        one,
        five,
        fifteen
    }

    pub label_enum UserType {
        user,
        other_bot,
        own,
    }

    pub label_enum EventType {
        cache_ready,
        channel_create,
        category_create,
        category_delete,
        channel_delete,
        channel_pins_update,
        channel_update,
        guild_ban_addition,
        guild_ban_removal,
        guild_create,
        guild_delete,
        guild_emojis_update,
        guild_integrations_update,
        guild_member_addition,
        guild_member_removal,
        guild_member_update,
        guild_members_chunk,
        guild_role_create,
        guild_role_delete,
        guild_role_update,
        guild_unavailable,
        guild_update,
        invite_create,
        invite_delete,
        message,
        message_delete,
        message_delete_bulk,
        message_update,
        reaction_add,
        reaction_remove,
        reaction_remove_all,
        presence_replace,
        presence_update,
        ready,
        resume,
        shard_stage_update,
        typing_start,
        unknown,
        user_update,
        voice_server_update,
        voice_state_update,
        webhook_update,
        interaction_create,
    }
    pub label_enum CommandsUsed {
        donate,
        help,
        join,
        language,
        ping,
        register_cmds,
        setup,
        train_storage
    }

    pub struct MessageCounterVec: IntCounter {
        "user_type" => UserType,
    }

    pub struct EventCounterVec: IntCounter {
        "event_type" => EventType,
    }

    pub struct CpuUsageVec: Gauge {
        "cpu_type" => CpuUsageType,
    }

    pub struct MemoryUsageVec: IntGauge {
        "memory_type" => MemoryUsageType,
    }

    pub struct BlockStatsVec: IntGauge {
        "disk_stats" => BlockStats,
    }

    pub struct SocketStatsVec: IntGauge {
        "socket_stats" => SocketStats,
    }

    pub struct NetworkStatsVec: IntGauge {
        "network_stats" => NetworkStats
    }

    pub struct LoadAvgStatsVec: Gauge {
        "load_avg" => LoadAvgStats,
    }

    pub struct CommandsUsedVec: IntCounter {
        "command_name" => CommandsUsed,
    }
}

static METRICS: OnceCell<Arc<Metrics>> = OnceCell::new();

pub struct Metrics {
    pub registry: Registry,
    pub start_time: NaiveDateTime,
    pub messages: MessageCounterVec,
    pub events: EventCounterVec,
    pub guilds: IntGauge,
    pub members: IntGauge,
    pub ms_transcribed: IntCounter,
    pub total_events: IntCounter,
    pub avg_audio_process_time: IntGauge,
    pub cpu_usage: CpuUsageVec,
    pub mem_usage: MemoryUsageVec,
    pub block_stats: BlockStatsVec,
    pub socket_stats: SocketStatsVec,
    pub network_stats: NetworkStatsVec,
    pub load_avg_stats: LoadAvgStatsVec,
    pub cpu_temp: Gauge,
    pub total_commands: IntCounter,
    pub commands: CommandsUsedVec,
}

impl Metrics {
    fn new() -> Arc<Self> {
        let registry = Registry::new_custom(Some("scripty".into()), None).unwrap();

        let messages_vec =
            IntCounterVec::new(Opts::new("messages", "Received messages"), &["user_type"]).unwrap();
        let messages_static_vec = MessageCounterVec::from(&messages_vec);
        registry.register(Box::new(messages_vec)).unwrap();

        let events_vec =
            IntCounterVec::new(Opts::new("events", "Gateway events"), &["event_type"]).unwrap();
        let events_static_vec = EventCounterVec::from(&events_vec);
        registry.register(Box::new(events_vec)).unwrap();

        let guilds_gauge = IntGauge::new("guilds", "Current guilds").unwrap();
        registry.register(Box::new(guilds_gauge.clone())).unwrap();

        let members_gauge = IntGauge::new("members", "Current members").unwrap();
        registry.register(Box::new(members_gauge.clone())).unwrap();

        let ms_transcribed =
            IntCounter::new("audio_transcribed", "Milliseconds of audio transcribed").unwrap();
        registry.register(Box::new(ms_transcribed.clone())).unwrap();

        let events = IntCounter::new("total_events", "Total gateway events").unwrap();
        registry.register(Box::new(events.clone())).unwrap();

        let audio_process = IntGauge::new(
            "avg_audio_process_time",
            "Average time to process one audio packet",
        )
        .unwrap();
        registry.register(Box::new(audio_process.clone())).unwrap();

        let cpu_usage = GaugeVec::new(Opts::new("cpu_usage", "CPU usage"), &["cpu_type"]).unwrap();
        let cpu_usage_static = CpuUsageVec::from(&cpu_usage);
        registry.register(Box::new(cpu_usage.clone())).unwrap();

        let mem_usage =
            IntGaugeVec::new(Opts::new("mem_usage", "Memory usage"), &["memory_type"]).unwrap();
        let mem_usage_static = MemoryUsageVec::from(&mem_usage);
        registry.register(Box::new(mem_usage.clone())).unwrap();

        let block_stats =
            IntGaugeVec::new(Opts::new("block_io", "Block statistics"), &["disk_stats"]).unwrap();
        let block_stats_static = BlockStatsVec::from(&block_stats);
        registry.register(Box::new(block_stats.clone())).unwrap();

        let load_avg =
            GaugeVec::new(Opts::new("load_avg", "Average system load"), &["load_avg"]).unwrap();
        let load_avg_static = LoadAvgStatsVec::from(&load_avg);
        registry.register(Box::new(load_avg.clone())).unwrap();

        let socket_stats =
            IntGaugeVec::new(Opts::new("socket_stats", "Socket stats"), &["socket_stats"]).unwrap();
        let socket_stats_static = SocketStatsVec::from(&socket_stats);
        registry.register(Box::new(socket_stats.clone())).unwrap();

        let net_stats =
            IntGaugeVec::new(Opts::new("net_stats", "Network stats"), &["network_stats"]).unwrap();
        let network_stats_static = NetworkStatsVec::from(&net_stats);
        registry.register(Box::new(net_stats.clone())).unwrap();

        let cpu_temp = Gauge::new("cpu_temp", "CPU temperature").unwrap();
        registry.register(Box::new(cpu_temp.clone())).unwrap();

        let total_commands_used =
            IntCounter::new("total_commands_used", "All commands used").unwrap();
        registry
            .register(Box::new(total_commands_used.clone()))
            .unwrap();

        let commands_used = IntCounterVec::new(
            Opts::new("commands_used", "Commands used"),
            &["command_name"],
        )
        .unwrap();
        let commands_used_static = CommandsUsedVec::from(&commands_used);
        registry.register(Box::new(commands_used.clone())).unwrap();

        Arc::new(Self {
            registry,
            start_time: Utc::now().naive_utc(),
            messages: messages_static_vec,
            events: events_static_vec,
            guilds: guilds_gauge,
            members: members_gauge,
            ms_transcribed,
            total_events: events,
            avg_audio_process_time: audio_process,
            cpu_usage: cpu_usage_static,
            mem_usage: mem_usage_static,
            block_stats: block_stats_static,
            socket_stats: socket_stats_static,
            network_stats: network_stats_static,
            load_avg_stats: load_avg_static,
            cpu_temp,
            total_commands: total_commands_used,
            commands: commands_used_static,
        })
    }
}