Next stuff
* FFT viewer - probably could be done with lines and minimum modifications
* Spectrum viewer - probably need to make data on CPU and upload to GPU, seems wasteful, I guess do bufferSubData and maybe a uniform for fragment shader to let you scrtoll through (that would be why theres those ones that scroll in place but I hate that)
* am I aliasing saw waves?
* tune saw waves to certain intervals eg third fifth
* are my detune cents really cents?
* integral sliders
* decimator / bit cruncher / saturator
    * even harmonics : think saw wave (has both), what about an oscillator thats just even? more analog, nice, musical, because octaves
    * ok all even is half wave rectified sinusioid, also f0 sin + 2f0 saw
    * odd harmonics: think square wave, digital
    * how do you make these, with decimation? can you make arbitrary ones with fft?
    * yea actually fuck this i got other stuff i can do. vcf is higher value,



harder:
* loop pedal, quantizer, always playback editor etc
* VCF will be good, filth time, recursive filters: we hope they converge

--

*nb definitely for bass partials are important - saturate it


not sure why the phase cancellation hits so ahrd at certain frequencies, seems like my detune could be wrong, unit test?

also not sure if aliasing, probably
is there tasteful use of aliasing?