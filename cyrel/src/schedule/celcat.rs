use chrono::NaiveDateTime;
use futures::future::try_join_all;

use super::Course;
use crate::celcat::{
    fetchable::{
        calendar::{self, CalView, CalendarData, CalendarDataRequest},
        event::{SideBarEvent, SideBarEventElement, SideBarEventRequest},
    },
    resource::resource_type::{self, WrapResourceType},
    Celcat,
};

pub use crate::celcat::resource::GroupId;

pub async fn fetch_calendar(
    start: NaiveDateTime,
    end: NaiveDateTime,
    group: GroupId,
) -> anyhow::Result<Vec<Course>> {
    let mut celcat = Celcat::new().await?;
    celcat.login().await?;

    let courses: Vec<calendar::Course> = celcat
        .fetch::<CalendarData<_>>(CalendarDataRequest {
            start,
            end,
            res_type: WrapResourceType(resource_type::Group),
            cal_view: CalView::Month,
            federation_ids: group,
            colour_scheme: 3,
        })
        .await?
        .courses;

    let mut futures = vec![];
    for c in courses {
        let f = async {
            celcat
                .fetch::<SideBarEvent>(SideBarEventRequest {
                    event_id: c.id.clone(),
                })
                .await
                .map(|s| (c, s))
        };
        futures.push(f);
    }

    let courses = try_join_all(futures).await?;

    courses
        .into_iter()
        .map(|(c, s)| side_bar_event_to_course(c, s))
        .collect()
}

fn side_bar_event_to_course(c: calendar::Course, s: SideBarEvent) -> anyhow::Result<Course> {
    let mut c = Course {
        id: c.id.0,
        start: c.start,
        end: c.end,
        category: None,
        module: None,
        room: None,
        teacher: None,
        description: None,
    };

    for e in s.elements {
        match e {
            SideBarEventElement::Category(cat) => {
                c.category = cat.content;
            }
            SideBarEventElement::Module(m) => {
                c.module = m.content;
            }
            SideBarEventElement::Room(r) => {
                c.room = r.content;
            }
            SideBarEventElement::Teacher(t) => {
                c.teacher = t.content;
            }
            SideBarEventElement::Name(d) => {
                c.description = d.content;
            }
            _ => (),
        }
    }

    Ok(c)
}
