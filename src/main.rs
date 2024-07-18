use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
enum TrafficLight {
    Red,
    Green,
}

#[derive(Clone)]
struct Direction {
    traffic_light: TrafficLight,
    cars: usize,
    connects_to: usize,
}

#[derive(Clone)]
struct Intersection {
    id: usize,
    north_to_south: Direction,
    south_to_north: Direction,
    east_to_west: Direction,
    west_to_east: Direction,
}

impl Intersection {
    fn new(id: usize, nts: Direction, stn: Direction, etw: Direction, wte: Direction) -> Self {
        Self {
            id,
            north_to_south: nts,
            south_to_north: stn,
            east_to_west: etw,
            west_to_east: wte,
        }
    }

    fn move_cars(direction: &mut Direction) -> bool {
        if let TrafficLight::Green = direction.traffic_light {
            if direction.cars > 0 {
                direction.cars -= 1;
                println!("Moved one car to intersection {}", direction.connects_to);
                return true;
            }
        }
        false
    }

    fn toggle_lights(&mut self) {
        match self.north_to_south.traffic_light {
            TrafficLight::Red => {
                self.north_to_south.traffic_light = TrafficLight::Green;
                self.south_to_north.traffic_light = TrafficLight::Green;
                self.east_to_west.traffic_light = TrafficLight::Red;
                self.west_to_east.traffic_light = TrafficLight::Red;
            }
            TrafficLight::Green => {
                self.north_to_south.traffic_light = TrafficLight::Red;
                self.south_to_north.traffic_light = TrafficLight::Red;
                self.east_to_west.traffic_light = TrafficLight::Green;
                self.west_to_east.traffic_light = TrafficLight::Green;
            }
        }
    }

    fn prioritize_direction(&self) -> bool {
        let north_south_total = self.north_to_south.cars + self.south_to_north.cars;
        let east_west_total = self.east_to_west.cars + self.west_to_east.cars;

        north_south_total >= east_west_total
    }

    fn toggle_lights_efficient(&mut self) {
        if self.prioritize_direction() {
            self.north_to_south.traffic_light = TrafficLight::Green;
            self.south_to_north.traffic_light = TrafficLight::Green;
            self.east_to_west.traffic_light = TrafficLight::Red;
            self.west_to_east.traffic_light = TrafficLight::Red;
        } else {
            self.north_to_south.traffic_light = TrafficLight::Red;
            self.south_to_north.traffic_light = TrafficLight::Red;
            self.east_to_west.traffic_light = TrafficLight::Green;
            self.west_to_east.traffic_light = TrafficLight::Green;
        }
    }
}

fn traffic_light_sequential(intersections: Arc<Mutex<Vec<Intersection>>>, running: Arc<Mutex<bool>>) {
    let light_handle = thread::spawn({
        let intersections = Arc::clone(&intersections);
        let running = Arc::clone(&running);
        move || {
            while *running.lock().unwrap() {
                thread::sleep(Duration::from_secs(2));
                let mut intersections = intersections.lock().unwrap();
                for intersection in intersections.iter_mut() {
                    intersection.toggle_lights();
                }
            }
        }
    });

    let mut handles = vec![];

    for i in 0..intersections.lock().unwrap().len() {
        let intersections = Arc::clone(&intersections);
        let running = Arc::clone(&running);
        let handle = thread::spawn(move || loop {
            let mut intersections = intersections.lock().unwrap();
            let mut running = running.lock().unwrap();
            let intersection = &mut intersections[i];

            let mut moved = false;
            moved |= Intersection::move_cars(&mut intersection.north_to_south);
            moved |= Intersection::move_cars(&mut intersection.south_to_north);
            moved |= Intersection::move_cars(&mut intersection.east_to_west);
            moved |= Intersection::move_cars(&mut intersection.west_to_east);

            if !moved {
                *running = false;
            }

            if !*running {
                break;
            }

            drop(intersections);
            drop(running);

            thread::sleep(Duration::from_secs(1));
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    *running.lock().unwrap() = false;
    light_handle.join().unwrap();
}

fn traffic_light_efficient(intersections: Arc<Mutex<Vec<Intersection>>>, running: Arc<Mutex<bool>>) {
    let light_handle = thread::spawn({
        let intersections = Arc::clone(&intersections);
        let running = Arc::clone(&running);
        move || {
            while *running.lock().unwrap() {
                let mut intersections = intersections.lock().unwrap();
                for intersection in intersections.iter_mut() {
                    let mut all_cars_moved = true;

                    if intersection.prioritize_direction() {
                        if intersection.north_to_south.cars > 0 || intersection.south_to_north.cars > 0 {
                            all_cars_moved = false;
                        }
                    } else {
                        if intersection.east_to_west.cars > 0 || intersection.west_to_east.cars > 0 {
                            all_cars_moved = false;
                        }
                    }

                    if all_cars_moved {
                        intersection.toggle_lights_efficient();
                    }
                }
            }
        }
    });

    let mut handles = vec![];

    for i in 0..intersections.lock().unwrap().len() {
        let intersections = Arc::clone(&intersections);
        let running = Arc::clone(&running);
        let handle = thread::spawn(move || loop {
            let mut intersections = intersections.lock().unwrap();
            let mut running = running.lock().unwrap();
            let intersection = &mut intersections[i];

            let mut moved = false;
            moved |= Intersection::move_cars(&mut intersection.north_to_south);
            moved |= Intersection::move_cars(&mut intersection.south_to_north);
            moved |= Intersection::move_cars(&mut intersection.east_to_west);
            moved |= Intersection::move_cars(&mut intersection.west_to_east);

            if !moved {
                *running = false;
            }

            if !*running {
                break;
            }

            drop(intersections);
            drop(running);

            thread::sleep(Duration::from_secs(1));
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    *running.lock().unwrap() = false;
    light_handle.join().unwrap();
}

fn main() {
    let start_time = Instant::now();

    let intersections = vec![
        Intersection::new(1, Direction { traffic_light: TrafficLight::Red, cars: 5, connects_to: 4 },
                             Direction { traffic_light: TrafficLight::Green, cars: 3, connects_to: 0 },
                             Direction { traffic_light: TrafficLight::Green, cars: 2, connects_to: 2 },
                             Direction { traffic_light: TrafficLight::Red, cars: 4, connects_to: 0 }),
        Intersection::new(2, Direction { traffic_light: TrafficLight::Green, cars: 6, connects_to: 5 },
                             Direction { traffic_light: TrafficLight::Red, cars: 1, connects_to: 0 },
                             Direction { traffic_light: TrafficLight::Green, cars: 3, connects_to: 3 },
                             Direction { traffic_light: TrafficLight::Red, cars: 2, connects_to: 1 }),
        Intersection::new(3, Direction { traffic_light: TrafficLight::Red, cars: 4, connects_to: 6 },
                             Direction { traffic_light: TrafficLight::Green, cars: 5, connects_to: 0 },
                             Direction { traffic_light: TrafficLight::Red, cars: 1, connects_to: 0 },
                             Direction { traffic_light: TrafficLight::Green, cars: 6, connects_to: 2 }),
        Intersection::new(4, Direction { traffic_light: TrafficLight::Red, cars: 3, connects_to: 7 },
                             Direction { traffic_light: TrafficLight::Green, cars: 4, connects_to: 1 },
                             Direction { traffic_light: TrafficLight::Green, cars: 2, connects_to: 5 },
                             Direction { traffic_light: TrafficLight::Red, cars: 1, connects_to: 0 }),
        Intersection::new(5, Direction { traffic_light: TrafficLight::Green, cars: 2, connects_to: 8 },
                             Direction { traffic_light: TrafficLight::Red, cars: 5, connects_to: 2 },
                             Direction { traffic_light: TrafficLight::Green, cars: 3, connects_to: 6 },
                             Direction { traffic_light: TrafficLight::Red, cars: 2, connects_to: 4 }),
        Intersection::new(6, Direction { traffic_light: TrafficLight::Red, cars: 4, connects_to: 9 },
                             Direction { traffic_light: TrafficLight::Green, cars: 6, connects_to: 3 },
                             Direction { traffic_light: TrafficLight::Red, cars: 2, connects_to: 0 },
                             Direction { traffic_light: TrafficLight::Green, cars: 5, connects_to: 5 }),
        Intersection::new(7, Direction { traffic_light: TrafficLight::Green, cars: 1, connects_to: 0 },
                             Direction { traffic_light: TrafficLight::Red, cars: 3, connects_to: 4 },
                             Direction { traffic_light: TrafficLight::Green, cars: 5, connects_to: 8 },
                             Direction { traffic_light: TrafficLight::Red, cars: 4, connects_to: 0 }),
        Intersection::new(8, Direction { traffic_light: TrafficLight::Red, cars: 6, connects_to: 0 },
                             Direction { traffic_light: TrafficLight::Green, cars: 2, connects_to: 5 },
                             Direction { traffic_light: TrafficLight::Green, cars: 3, connects_to: 9 },
                             Direction { traffic_light: TrafficLight::Red, cars: 1, connects_to: 7 }),
        Intersection::new(9, Direction { traffic_light: TrafficLight::Red, cars: 2, connects_to: 0 },
                             Direction { traffic_light: TrafficLight::Green, cars: 4, connects_to: 6 },
                             Direction { traffic_light: TrafficLight::Red, cars: 5, connects_to: 0 },
                             Direction { traffic_light: TrafficLight::Green, cars: 6, connects_to: 8 }),
    ];

    let intersections = Arc::new(Mutex::new(intersections));
    let running = Arc::new(Mutex::new(true));
    
    traffic_light_sequential(Arc::clone(&intersections), Arc::clone(&running));
    // traffic_light_efficient(Arc::clone(&intersections), Arc::clone(&running));

    let duration = start_time.elapsed();
    println!("Time taken: {:?}", duration);
}
