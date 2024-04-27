//! A light-emitting object in space

use bevy::{
    prelude::{App, Color, Component},
    reflect::Reflect,
};
use serde::{Deserialize, Serialize};

/// Taken from http://www.vendian.org/mncharity/dir3/blackbody/UnstableURLs/bbr_color.html
// Clippy thinks some of these are random constants
#[allow(clippy::approx_constant)]
const COLOR_TABLE: [[f32; 3]; 391] = [
    [1.0000, 0.0337, 0.0000],
    [1.0000, 0.0592, 0.0000],
    [1.0000, 0.0846, 0.0000],
    [1.0000, 0.1096, 0.0000],
    [1.0000, 0.1341, 0.0000],
    [1.0000, 0.1578, 0.0000],
    [1.0000, 0.1806, 0.0000],
    [1.0000, 0.2025, 0.0000],
    [1.0000, 0.2235, 0.0000],
    [1.0000, 0.2434, 0.0000],
    [1.0000, 0.2647, 0.0033],
    [1.0000, 0.2889, 0.0120],
    [1.0000, 0.3126, 0.0219],
    [1.0000, 0.3360, 0.0331],
    [1.0000, 0.3589, 0.0454],
    [1.0000, 0.3814, 0.0588],
    [1.0000, 0.4034, 0.0734],
    [1.0000, 0.4250, 0.0889],
    [1.0000, 0.4461, 0.1054],
    [1.0000, 0.4668, 0.1229],
    [1.0000, 0.4870, 0.1411],
    [1.0000, 0.5067, 0.1602],
    [1.0000, 0.5259, 0.1800],
    [1.0000, 0.5447, 0.2005],
    [1.0000, 0.5630, 0.2216],
    [1.0000, 0.5809, 0.2433],
    [1.0000, 0.5983, 0.2655],
    [1.0000, 0.6153, 0.2881],
    [1.0000, 0.6318, 0.3112],
    [1.0000, 0.6480, 0.3346],
    [1.0000, 0.6636, 0.3583],
    [1.0000, 0.6789, 0.3823],
    [1.0000, 0.6938, 0.4066],
    [1.0000, 0.7083, 0.4310],
    [1.0000, 0.7223, 0.4556],
    [1.0000, 0.7360, 0.4803],
    [1.0000, 0.7494, 0.5051],
    [1.0000, 0.7623, 0.5299],
    [1.0000, 0.7750, 0.5548],
    [1.0000, 0.7872, 0.5797],
    [1.0000, 0.7992, 0.6045],
    [1.0000, 0.8108, 0.6293],
    [1.0000, 0.8221, 0.6541],
    [1.0000, 0.8330, 0.6787],
    [1.0000, 0.8437, 0.7032],
    [1.0000, 0.8541, 0.7277],
    [1.0000, 0.8642, 0.7519],
    [1.0000, 0.8740, 0.7760],
    [1.0000, 0.8836, 0.8000],
    [1.0000, 0.8929, 0.8238],
    [1.0000, 0.9019, 0.8473],
    [1.0000, 0.9107, 0.8707],
    [1.0000, 0.9193, 0.8939],
    [1.0000, 0.9276, 0.9168],
    [1.0000, 0.9357, 0.9396],
    [1.0000, 0.9436, 0.9621],
    [1.0000, 0.9513, 0.9844],
    [0.9937, 0.9526, 1.0000],
    [0.9726, 0.9395, 1.0000],
    [0.9526, 0.9270, 1.0000],
    [0.9337, 0.9150, 1.0000],
    [0.9157, 0.9035, 1.0000],
    [0.8986, 0.8925, 1.0000],
    [0.8823, 0.8819, 1.0000],
    [0.8668, 0.8718, 1.0000],
    [0.8520, 0.8621, 1.0000],
    [0.8379, 0.8527, 1.0000],
    [0.8244, 0.8437, 1.0000],
    [0.8115, 0.8351, 1.0000],
    [0.7992, 0.8268, 1.0000],
    [0.7874, 0.8187, 1.0000],
    [0.7761, 0.8110, 1.0000],
    [0.7652, 0.8035, 1.0000],
    [0.7548, 0.7963, 1.0000],
    [0.7449, 0.7894, 1.0000],
    [0.7353, 0.7827, 1.0000],
    [0.7260, 0.7762, 1.0000],
    [0.7172, 0.7699, 1.0000],
    [0.7086, 0.7638, 1.0000],
    [0.7004, 0.7579, 1.0000],
    [0.6925, 0.7522, 1.0000],
    [0.6848, 0.7467, 1.0000],
    [0.6774, 0.7414, 1.0000],
    [0.6703, 0.7362, 1.0000],
    [0.6635, 0.7311, 1.0000],
    [0.6568, 0.7263, 1.0000],
    [0.6504, 0.7215, 1.0000],
    [0.6442, 0.7169, 1.0000],
    [0.6382, 0.7124, 1.0000],
    [0.6324, 0.7081, 1.0000],
    [0.6268, 0.7039, 1.0000],
    [0.6213, 0.6998, 1.0000],
    [0.6161, 0.6958, 1.0000],
    [0.6109, 0.6919, 1.0000],
    [0.6060, 0.6881, 1.0000],
    [0.6012, 0.6844, 1.0000],
    [0.5965, 0.6808, 1.0000],
    [0.5919, 0.6773, 1.0000],
    [0.5875, 0.6739, 1.0000],
    [0.5833, 0.6706, 1.0000],
    [0.5791, 0.6674, 1.0000],
    [0.5750, 0.6642, 1.0000],
    [0.5711, 0.6611, 1.0000],
    [0.5673, 0.6581, 1.0000],
    [0.5636, 0.6552, 1.0000],
    [0.5599, 0.6523, 1.0000],
    [0.5564, 0.6495, 1.0000],
    [0.5530, 0.6468, 1.0000],
    [0.5496, 0.6441, 1.0000],
    [0.5463, 0.6415, 1.0000],
    [0.5431, 0.6389, 1.0000],
    [0.5400, 0.6364, 1.0000],
    [0.5370, 0.6340, 1.0000],
    [0.5340, 0.6316, 1.0000],
    [0.5312, 0.6293, 1.0000],
    [0.5283, 0.6270, 1.0000],
    [0.5256, 0.6247, 1.0000],
    [0.5229, 0.6225, 1.0000],
    [0.5203, 0.6204, 1.0000],
    [0.5177, 0.6183, 1.0000],
    [0.5152, 0.6162, 1.0000],
    [0.5128, 0.6142, 1.0000],
    [0.5104, 0.6122, 1.0000],
    [0.5080, 0.6103, 1.0000],
    [0.5057, 0.6084, 1.0000],
    [0.5035, 0.6065, 1.0000],
    [0.5013, 0.6047, 1.0000],
    [0.4991, 0.6029, 1.0000],
    [0.4970, 0.6012, 1.0000],
    [0.4950, 0.5994, 1.0000],
    [0.4930, 0.5978, 1.0000],
    [0.4910, 0.5961, 1.0000],
    [0.4891, 0.5945, 1.0000],
    [0.4872, 0.5929, 1.0000],
    [0.4853, 0.5913, 1.0000],
    [0.4835, 0.5898, 1.0000],
    [0.4817, 0.5882, 1.0000],
    [0.4799, 0.5868, 1.0000],
    [0.4782, 0.5853, 1.0000],
    [0.4765, 0.5839, 1.0000],
    [0.4749, 0.5824, 1.0000],
    [0.4733, 0.5811, 1.0000],
    [0.4717, 0.5797, 1.0000],
    [0.4701, 0.5784, 1.0000],
    [0.4686, 0.5770, 1.0000],
    [0.4671, 0.5757, 1.0000],
    [0.4656, 0.5745, 1.0000],
    [0.4641, 0.5732, 1.0000],
    [0.4627, 0.5720, 1.0000],
    [0.4613, 0.5708, 1.0000],
    [0.4599, 0.5696, 1.0000],
    [0.4586, 0.5684, 1.0000],
    [0.4572, 0.5673, 1.0000],
    [0.4559, 0.5661, 1.0000],
    [0.4546, 0.5650, 1.0000],
    [0.4534, 0.5639, 1.0000],
    [0.4521, 0.5628, 1.0000],
    [0.4509, 0.5617, 1.0000],
    [0.4497, 0.5607, 1.0000],
    [0.4485, 0.5597, 1.0000],
    [0.4474, 0.5586, 1.0000],
    [0.4462, 0.5576, 1.0000],
    [0.4451, 0.5566, 1.0000],
    [0.4440, 0.5557, 1.0000],
    [0.4429, 0.5547, 1.0000],
    [0.4418, 0.5538, 1.0000],
    [0.4408, 0.5528, 1.0000],
    [0.4397, 0.5519, 1.0000],
    [0.4387, 0.5510, 1.0000],
    [0.4377, 0.5501, 1.0000],
    [0.4367, 0.5492, 1.0000],
    [0.4357, 0.5483, 1.0000],
    [0.4348, 0.5475, 1.0000],
    [0.4338, 0.5466, 1.0000],
    [0.4329, 0.5458, 1.0000],
    [0.4319, 0.5450, 1.0000],
    [0.4310, 0.5442, 1.0000],
    [0.4301, 0.5434, 1.0000],
    [0.4293, 0.5426, 1.0000],
    [0.4284, 0.5418, 1.0000],
    [0.4275, 0.5410, 1.0000],
    [0.4267, 0.5403, 1.0000],
    [0.4258, 0.5395, 1.0000],
    [0.4250, 0.5388, 1.0000],
    [0.4242, 0.5381, 1.0000],
    [0.4234, 0.5373, 1.0000],
    [0.4226, 0.5366, 1.0000],
    [0.4218, 0.5359, 1.0000],
    [0.4211, 0.5352, 1.0000],
    [0.4203, 0.5345, 1.0000],
    [0.4196, 0.5339, 1.0000],
    [0.4188, 0.5332, 1.0000],
    [0.4181, 0.5325, 1.0000],
    [0.4174, 0.5319, 1.0000],
    [0.4167, 0.5312, 1.0000],
    [0.4160, 0.5306, 1.0000],
    [0.4153, 0.5300, 1.0000],
    [0.4146, 0.5293, 1.0000],
    [0.4139, 0.5287, 1.0000],
    [0.4133, 0.5281, 1.0000],
    [0.4126, 0.5275, 1.0000],
    [0.4119, 0.5269, 1.0000],
    [0.4113, 0.5264, 1.0000],
    [0.4107, 0.5258, 1.0000],
    [0.4100, 0.5252, 1.0000],
    [0.4094, 0.5246, 1.0000],
    [0.4088, 0.5241, 1.0000],
    [0.4082, 0.5235, 1.0000],
    [0.4076, 0.5230, 1.0000],
    [0.4070, 0.5224, 1.0000],
    [0.4064, 0.5219, 1.0000],
    [0.4059, 0.5214, 1.0000],
    [0.4053, 0.5209, 1.0000],
    [0.4047, 0.5203, 1.0000],
    [0.4042, 0.5198, 1.0000],
    [0.4036, 0.5193, 1.0000],
    [0.4031, 0.5188, 1.0000],
    [0.4026, 0.5183, 1.0000],
    [0.4020, 0.5178, 1.0000],
    [0.4015, 0.5174, 1.0000],
    [0.4010, 0.5169, 1.0000],
    [0.4005, 0.5164, 1.0000],
    [0.4000, 0.5159, 1.0000],
    [0.3995, 0.5155, 1.0000],
    [0.3990, 0.5150, 1.0000],
    [0.3985, 0.5146, 1.0000],
    [0.3980, 0.5141, 1.0000],
    [0.3975, 0.5137, 1.0000],
    [0.3970, 0.5132, 1.0000],
    [0.3966, 0.5128, 1.0000],
    [0.3961, 0.5123, 1.0000],
    [0.3956, 0.5119, 1.0000],
    [0.3952, 0.5115, 1.0000],
    [0.3947, 0.5111, 1.0000],
    [0.3943, 0.5107, 1.0000],
    [0.3938, 0.5103, 1.0000],
    [0.3934, 0.5098, 1.0000],
    [0.3930, 0.5094, 1.0000],
    [0.3925, 0.5090, 1.0000],
    [0.3921, 0.5086, 1.0000],
    [0.3917, 0.5083, 1.0000],
    [0.3913, 0.5079, 1.0000],
    [0.3909, 0.5075, 1.0000],
    [0.3905, 0.5071, 1.0000],
    [0.3901, 0.5067, 1.0000],
    [0.3897, 0.5064, 1.0000],
    [0.3893, 0.5060, 1.0000],
    [0.3889, 0.5056, 1.0000],
    [0.3885, 0.5053, 1.0000],
    [0.3881, 0.5049, 1.0000],
    [0.3877, 0.5045, 1.0000],
    [0.3874, 0.5042, 1.0000],
    [0.3870, 0.5038, 1.0000],
    [0.3866, 0.5035, 1.0000],
    [0.3863, 0.5032, 1.0000],
    [0.3859, 0.5028, 1.0000],
    [0.3855, 0.5025, 1.0000],
    [0.3852, 0.5021, 1.0000],
    [0.3848, 0.5018, 1.0000],
    [0.3845, 0.5015, 1.0000],
    [0.3841, 0.5012, 1.0000],
    [0.3838, 0.5008, 1.0000],
    [0.3835, 0.5005, 1.0000],
    [0.3831, 0.5002, 1.0000],
    [0.3828, 0.4999, 1.0000],
    [0.3825, 0.4996, 1.0000],
    [0.3821, 0.4993, 1.0000],
    [0.3818, 0.4990, 1.0000],
    [0.3815, 0.4987, 1.0000],
    [0.3812, 0.4984, 1.0000],
    [0.3809, 0.4981, 1.0000],
    [0.3805, 0.4978, 1.0000],
    [0.3802, 0.4975, 1.0000],
    [0.3799, 0.4972, 1.0000],
    [0.3796, 0.4969, 1.0000],
    [0.3793, 0.4966, 1.0000],
    [0.3790, 0.4963, 1.0000],
    [0.3787, 0.4960, 1.0000],
    [0.3784, 0.4958, 1.0000],
    [0.3781, 0.4955, 1.0000],
    [0.3779, 0.4952, 1.0000],
    [0.3776, 0.4949, 1.0000],
    [0.3773, 0.4947, 1.0000],
    [0.3770, 0.4944, 1.0000],
    [0.3767, 0.4941, 1.0000],
    [0.3764, 0.4939, 1.0000],
    [0.3762, 0.4936, 1.0000],
    [0.3759, 0.4934, 1.0000],
    [0.3756, 0.4931, 1.0000],
    [0.3754, 0.4928, 1.0000],
    [0.3751, 0.4926, 1.0000],
    [0.3748, 0.4923, 1.0000],
    [0.3746, 0.4921, 1.0000],
    [0.3743, 0.4918, 1.0000],
    [0.3741, 0.4916, 1.0000],
    [0.3738, 0.4914, 1.0000],
    [0.3735, 0.4911, 1.0000],
    [0.3733, 0.4909, 1.0000],
    [0.3730, 0.4906, 1.0000],
    [0.3728, 0.4904, 1.0000],
    [0.3726, 0.4902, 1.0000],
    [0.3723, 0.4899, 1.0000],
    [0.3721, 0.4897, 1.0000],
    [0.3718, 0.4895, 1.0000],
    [0.3716, 0.4893, 1.0000],
    [0.3714, 0.4890, 1.0000],
    [0.3711, 0.4888, 1.0000],
    [0.3709, 0.4886, 1.0000],
    [0.3707, 0.4884, 1.0000],
    [0.3704, 0.4881, 1.0000],
    [0.3702, 0.4879, 1.0000],
    [0.3700, 0.4877, 1.0000],
    [0.3698, 0.4875, 1.0000],
    [0.3695, 0.4873, 1.0000],
    [0.3693, 0.4871, 1.0000],
    [0.3691, 0.4869, 1.0000],
    [0.3689, 0.4867, 1.0000],
    [0.3687, 0.4864, 1.0000],
    [0.3684, 0.4862, 1.0000],
    [0.3682, 0.4860, 1.0000],
    [0.3680, 0.4858, 1.0000],
    [0.3678, 0.4856, 1.0000],
    [0.3676, 0.4854, 1.0000],
    [0.3674, 0.4852, 1.0000],
    [0.3672, 0.4850, 1.0000],
    [0.3670, 0.4848, 1.0000],
    [0.3668, 0.4847, 1.0000],
    [0.3666, 0.4845, 1.0000],
    [0.3664, 0.4843, 1.0000],
    [0.3662, 0.4841, 1.0000],
    [0.3660, 0.4839, 1.0000],
    [0.3658, 0.4837, 1.0000],
    [0.3656, 0.4835, 1.0000],
    [0.3654, 0.4833, 1.0000],
    [0.3652, 0.4831, 1.0000],
    [0.3650, 0.4830, 1.0000],
    [0.3649, 0.4828, 1.0000],
    [0.3647, 0.4826, 1.0000],
    [0.3645, 0.4824, 1.0000],
    [0.3643, 0.4822, 1.0000],
    [0.3641, 0.4821, 1.0000],
    [0.3639, 0.4819, 1.0000],
    [0.3638, 0.4817, 1.0000],
    [0.3636, 0.4815, 1.0000],
    [0.3634, 0.4814, 1.0000],
    [0.3632, 0.4812, 1.0000],
    [0.3630, 0.4810, 1.0000],
    [0.3629, 0.4809, 1.0000],
    [0.3627, 0.4807, 1.0000],
    [0.3625, 0.4805, 1.0000],
    [0.3624, 0.4804, 1.0000],
    [0.3622, 0.4802, 1.0000],
    [0.3620, 0.4800, 1.0000],
    [0.3619, 0.4799, 1.0000],
    [0.3617, 0.4797, 1.0000],
    [0.3615, 0.4796, 1.0000],
    [0.3614, 0.4794, 1.0000],
    [0.3612, 0.4792, 1.0000],
    [0.3610, 0.4791, 1.0000],
    [0.3609, 0.4789, 1.0000],
    [0.3607, 0.4788, 1.0000],
    [0.3605, 0.4786, 1.0000],
    [0.3604, 0.4785, 1.0000],
    [0.3602, 0.4783, 1.0000],
    [0.3601, 0.4782, 1.0000],
    [0.3599, 0.4780, 1.0000],
    [0.3598, 0.4779, 1.0000],
    [0.3596, 0.4777, 1.0000],
    [0.3595, 0.4776, 1.0000],
    [0.3593, 0.4774, 1.0000],
    [0.3592, 0.4773, 1.0000],
    [0.3590, 0.4771, 1.0000],
    [0.3589, 0.4770, 1.0000],
    [0.3587, 0.4768, 1.0000],
    [0.3586, 0.4767, 1.0000],
    [0.3584, 0.4766, 1.0000],
    [0.3583, 0.4764, 1.0000],
    [0.3581, 0.4763, 1.0000],
    [0.3580, 0.4761, 1.0000],
    [0.3579, 0.4760, 1.0000],
    [0.3577, 0.4759, 1.0000],
    [0.3576, 0.4757, 1.0000],
    [0.3574, 0.4756, 1.0000],
    [0.3573, 0.4755, 1.0000],
    [0.3572, 0.4753, 1.0000],
    [0.3570, 0.4752, 1.0000],
    [0.3569, 0.4751, 1.0000],
    [0.3567, 0.4749, 1.0000],
    [0.3566, 0.4748, 1.0000],
    [0.3565, 0.4747, 1.0000],
    [0.3563, 0.4745, 1.0000],
];

#[derive(Debug, Component, Reflect, Serialize, Deserialize, Clone, Copy)]
/// Represents a light-emitting object in space.
///
/// This should be at the center of systems & are not saved/loaded to the disk
pub struct Star {
    temperature: f32,
}

/// The minimum temperature a star can be
pub const MIN_TEMPERATURE: f32 = 1_000.0;
/// The maximum temperature a star can be
pub const MAX_TEMPERATURE: f32 = 40_000.0;

impl Star {
    /// Creates a new star with the given temperature.
    ///
    /// * `temperature` In Kelvin. Our sun is 5,772.0K. This is clamped to be within `MIN_TEMPERATURE` and `MAX_TEMPERATURE`.
    pub fn new(temperature: f32) -> Self {
        let temperature = temperature.clamp(MIN_TEMPERATURE, MAX_TEMPERATURE);

        Self { temperature }
    }

    /// Gets the color this star should be.
    ///
    /// This is based on the star's temperature. A chart can be found here:
    /// http://www.vendian.org/mncharity/dir3/blackbody/UnstableURLs/bbr_color.html
    pub fn color(&self) -> Color {
        let temp_index = ((self.temperature - 1000.0) / 100.0) as usize;

        let rgb = COLOR_TABLE[temp_index];

        Color::rgb(rgb[0] * 20.0, rgb[1] * 20.0, rgb[2] * 20.0)
    }

    /// Gets this star's temperature in Kelvin
    pub fn temperature(&self) -> f32 {
        self.temperature
    }
}

pub(super) fn register(app: &mut App) {
    app.register_type::<Star>();
}
