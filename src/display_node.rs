use std::borrow::Cow;
use std::cell::RefCell;
use std::path::PathBuf;

use prometheus_client::collector::Collector;
use prometheus_client::encoding::{EncodeMetric as _, MetricEncoder};
use prometheus_client::metrics::MetricType;
use prometheus_client::metrics::gauge::ConstGauge;
use prometheus_client::registry::{Registry, Unit};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

use crate::display::{get_printable_name, human_readable_number};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct DisplayNode {
    // Note: the order of fields in important here, for PartialEq and PartialOrd
    pub size: u64,
    pub name: PathBuf,
    pub children: Vec<DisplayNode>,
}

impl DisplayNode {
    pub fn num_siblings(&self) -> u64 {
        self.children.len() as u64
    }

    pub fn get_children_from_node(&self, is_reversed: bool) -> impl Iterator<Item = &DisplayNode> {
        // we box to avoid the clippy lint warning
        let out: Box<dyn Iterator<Item = &DisplayNode>> = if is_reversed {
            Box::new(self.children.iter().rev())
        } else {
            Box::new(self.children.iter())
        };
        out
    }

    pub fn into_metrics(
        self,
        cli_params: Option<Vec<String>>,
        by_filecount: bool,
        skip_total: bool,
    ) -> String {
        #[derive(Debug)]
        struct DustExporter {
            root_node: DisplayNode,
            by_filecount: bool,
            skip_total: bool,
        }
        impl Collector for DustExporter {
            fn encode(
                &self,
                mut enc: prometheus_client::encoding::DescriptorEncoder,
            ) -> Result<(), std::fmt::Error> {
                let mut metric_encoder = match self.by_filecount {
                    false => enc.encode_descriptor(
                        "dust_file_size",
                        "Total size of files in this folder / size of this file.",
                        Some(&Unit::Bytes),
                        MetricType::Gauge,
                    )?,
                    true => enc.encode_descriptor(
                        "dust_file_count",
                        "Total number of files in this folder / '1' for files.",
                        None,
                        MetricType::Gauge,
                    )?,
                };
                self.root_node
                    .encode_metrics(&mut metric_encoder, self.skip_total)?;
                Ok(())
            }
        }

        let global_labels = cli_params_to_label(cli_params.as_ref());

        let mut registry = Registry::with_labels(global_labels.into_iter());
        registry.register_collector(Box::new(DustExporter {
            root_node: self,
            by_filecount,
            skip_total,
        }));

        let mut out = String::new();
        prometheus_client::encoding::text::encode(&mut out, &registry)
            .expect("String's Write impl never fails");
        out
    }
    fn encode_metrics(
        &self,
        metric_encoder: &mut MetricEncoder,
        skip_self: bool,
    ) -> Result<(), std::fmt::Error> {
        if !skip_self {
            let g = ConstGauge::new(self.size);
            let labels = [("path", get_printable_name(&self.name, false))];
            let labeled = metric_encoder.encode_family(&labels)?;
            g.encode(labeled)?;
        }
        for child in self.children.iter() {
            child.encode_metrics(metric_encoder, false)?;
        }
        Ok(())
    }
}

fn cli_params_to_label(
    cli_params: Option<&Vec<String>>,
) -> Option<(Cow<'static, str>, Cow<'static, str>)> {
    let params = cli_params?;

    let value = params.iter().fold(None, |acc, param| match acc {
        Some(acc) => Some(acc + " " + param.as_str()),
        None => Some(param.to_string()),
    })?;

    Some((Cow::Borrowed("paths"), Cow::Owned(value)))
}

// Only used for -j 'json' flag combined with -o 'output_type' flag
// Used to pass the output_type into the custom Serde serializer
thread_local! {
    pub static OUTPUT_TYPE: RefCell<String> = const { RefCell::new(String::new()) };
}

/*
We need the custom Serialize incase someone uses the -o flag to pass a custom output type in
(show size in Mb / Gb etc).
Sadly this also necessitates a global variable OUTPUT_TYPE as we can not pass the output_type flag
into the serialize method
 */
impl Serialize for DisplayNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let readable_size = OUTPUT_TYPE
            .with(|output_type| human_readable_number(self.size, output_type.borrow().as_str()));
        let mut state = serializer.serialize_struct("DisplayNode", 2)?;
        state.serialize_field("size", &(readable_size))?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("children", &self.children)?;
        state.end()
    }
}
