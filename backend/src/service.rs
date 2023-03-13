use crate::firefly::{
    backend_server::Backend, service_client::ServiceClient, Ack, Empty, Gradient, GradientWStrips,
    Rgb, RgbWStrips, StateWStrip, States, Strip as BackendStrip, StripIds, Strips, White,
    WhiteWStrips,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{transport::Channel, Code, Request, Response, Status};

#[derive(Debug)]
pub struct BackendService {}

#[tonic::async_trait]
impl Backend for BackendService {
    async fn on(
        &self,
        request: tonic::Request<StripIds>,
    ) -> Result<tonic::Response<Ack>, tonic::Status> {
        let mut state = request
            .extensions()
            .get::<Arc<Mutex<BackendState>>>()
            .ok_or_else(|| Status::new(Code::Internal, "Failed getting strips."))?
            .lock()
            .await;

        let requested_strips = &request.get_ref().strips;

        for strip in &mut state.strips {
            if requested_strips.contains(&(strip.id as i32)) {
                strip.client.on(Request::new(Empty {})).await?;
            }
        }

        Ok(Response::new(Ack { status: true }))
    }

    async fn off(
        &self,
        request: tonic::Request<StripIds>,
    ) -> Result<tonic::Response<Ack>, tonic::Status> {
        let mut state = request
            .extensions()
            .get::<Arc<Mutex<BackendState>>>()
            .ok_or_else(|| Status::new(Code::Internal, "Failed getting strips."))?
            .lock()
            .await;

        let requested_strips = &request.get_ref().strips;

        for strip in &mut state.strips {
            if requested_strips.contains(&(strip.id as i32)) {
                strip.client.off(Request::new(Empty {})).await?;
            }
        }

        Ok(Response::new(Ack { status: true }))
    }

    async fn set_rgb(
        &self,
        request: tonic::Request<RgbWStrips>,
    ) -> Result<tonic::Response<Ack>, tonic::Status> {
        let mut state = request
            .extensions()
            .get::<Arc<Mutex<BackendState>>>()
            .ok_or_else(|| Status::new(Code::Internal, "Failed getting strips."))?
            .lock()
            .await;

        let request_data = &request.get_ref();

        for strip in &mut state.strips {
            if request_data.strips.contains(&(strip.id as i32)) {
                strip
                    .client
                    .set_rgb(Request::new(Rgb {
                        red: request_data.red,
                        green: request_data.green,
                        blue: request_data.blue,
                    }))
                    .await?;
            }
        }

        Ok(Response::new(Ack { status: true }))
    }

    async fn set_white(
        &self,
        request: tonic::Request<WhiteWStrips>,
    ) -> Result<tonic::Response<Ack>, tonic::Status> {
        let mut state = request
            .extensions()
            .get::<Arc<Mutex<BackendState>>>()
            .ok_or_else(|| Status::new(Code::Internal, "Failed getting strips."))?
            .lock()
            .await;

        let request_data = &request.get_ref();

        for strip in &mut state.strips {
            if request_data.strips.contains(&(strip.id as i32)) {
                strip
                    .client
                    .set_white(Request::new(White {
                        white: request_data.white,
                    }))
                    .await?;
            }
        }

        Ok(Response::new(Ack { status: true }))
    }

    async fn set_gradient(
        &self,
        request: tonic::Request<GradientWStrips>,
    ) -> Result<tonic::Response<Ack>, tonic::Status> {
        let mut state = request
            .extensions()
            .get::<Arc<Mutex<BackendState>>>()
            .ok_or_else(|| Status::new(Code::Internal, "Failed getting strips."))?
            .lock()
            .await;

        let request_data = &request.get_ref();

        for strip in &mut state.strips {
            if request_data.strips.contains(&(strip.id as i32)) {
                strip
                    .client
                    .set_gradient(Request::new(Gradient {
                        colors: request_data.colors.clone(),
                    }))
                    .await?;
            }
        }

        Ok(Response::new(Ack { status: true }))
    }

    async fn get_state(
        &self,
        request: tonic::Request<StripIds>,
    ) -> Result<tonic::Response<States>, tonic::Status> {
        let mut state = request
            .extensions()
            .get::<Arc<Mutex<BackendState>>>()
            .ok_or_else(|| Status::new(Code::Internal, "Failed getting strips."))?
            .lock()
            .await;

        let strip_ids = &request.get_ref().strips;

        let mut states: Vec<StateWStrip> = Vec::new();

        for strip in &mut state.strips {
            if strip_ids.contains(&(strip.id as i32)) {
                let state = strip.client.get_state(Request::new(Empty {})).await?;
                states.push(StateWStrip {
                    strip: strip.id as i32,
                    on: state.get_ref().on,
                    leds: state.get_ref().leds.clone(),
                })
            }
        }

        Ok(Response::new(States { states }))
    }

    async fn get_strips(
        &self,
        request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<Strips>, tonic::Status> {
        let mut state = request
            .extensions()
            .get::<Arc<Mutex<BackendState>>>()
            .ok_or_else(|| Status::new(Code::Internal, "Failed getting strips."))?
            .lock()
            .await;

        let mut strips: Vec<BackendStrip> = Vec::new();

        for strip in &mut state.strips {
            strips.push(BackendStrip {
                id: strip.id as i32,
                name: strip.name.clone(),
            })
        }

        Ok(Response::new(Strips { strips }))
    }
}

pub struct BackendState {
    pub strips: Vec<Strip>,
}

#[derive(Debug)]
pub struct Strip {
    pub id: u32,
    pub name: String,
    pub client: ServiceClient<Channel>,
}
