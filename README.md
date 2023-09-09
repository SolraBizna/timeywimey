This is a simple utility that can be used to diagnose and analyze a faulty real-time clock.

# How to use

Install this utility on both the patient machine and a known-working diagnosing machine:

```
cargo install timeywimey
```

## Patient

On the patient, simply run `timeywimey server`. You can change the port or interface it listens on using `-a`, if you like. For maximum data, disable any network time sync your operating system might be performing in the background. (Don't forget to turn it back on afterward!)

## Diagnoser

On the diagnosing machine, run `timeywimey client -a ADDRESS_AND_PORT_OF_PATIENT -l path/to/logfile.csv`. Optionally, change the poll interval using `-p`. At the default poll interval of 5 seconds, the logfile will grow by about 600KB per day.

## Analysis

After letting it run for a few hours or days, plug the CSV file into your favorite spreadsheet, graphing, or analysis package. The first column is the UNIX timestamp on the diagnosing machine at the time the poll request was sent. The second column is difference between that timestamp and the timestamp reported by the patient machine. The third column is the diagnosing machine's measurement of the time, in seconds, that passed between the request being sent and the response being received.

Note that the logfile will have quite a lot of rows, and may make your spreadsheet package cry! (Try a longer poll interval if this happens.)

If you have the first column as the X axis and second column as the Y axis, you will see the trend of the time differential between the two machines. If everyone's RTCs are working well, the time differential will remain constant with very little drift (even one percent per day is heinous). If somebody's RTC is misbehaving, there will be a noticeable slope on the Y axis, possibly including discontinuities.

It may be interesting to try to make sense of changes in the slope of the differential. For instance, if it noticeably speeds up or slows down when intense gaming is being performed on the patient machine, that may indicate that the temperature compensation circuitry in that machine's RTC is no longer working. Or, if it often jumps by a fixed amount at a fixed interval, it may indicate a bad flip-flop in a counter or register somewhere.

# Security

Someone who can reach the port the server is listening on can get roughly 2:1 bandwidth amplification out of the deal. Other than that, security considerations are minimal. Still best not to expose it to the public Internet.

# Caveats

Due to laziness, this only measures UNIX time with one second of precision. This is a limitation imposed by using the quickest and most portable way to get RTC time.

# Legalese

TimeyWimey is copyright 2023, Solra Bizna, and licensed under either of:

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or
   <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the TimeyWimey crate by you, as defined
in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
