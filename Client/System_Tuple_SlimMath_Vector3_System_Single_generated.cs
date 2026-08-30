using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.System_Tuple_SlimMath_Vector3_System_Single
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(System.Tuple<SlimMath.Vector3,System.Single>); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (System.Tuple<SlimMath.Vector3,System.Single>)obj;
            s.Write(value.Item1);
            s.Write(value.Item2);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            SlimMath.Vector3 tmp0 = default(SlimMath.Vector3);
            s.Read(out tmp0);
            System.Single tmp1 = default(System.Single);
            s.Read(out tmp1);
            return new System.Tuple<SlimMath.Vector3,System.Single>(tmp0, tmp1);

        }
        
    }
}
