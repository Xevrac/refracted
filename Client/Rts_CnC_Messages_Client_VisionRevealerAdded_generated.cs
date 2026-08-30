using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_VisionRevealerAdded
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.VisionRevealerAdded); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.VisionRevealerAdded)obj;
            //  Serialize Id
            s.Write(value.Id);
            //  Serialize Position
            s.Write(value.Position);
            //  Serialize Radius
            s.Write(value.Radius);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.VisionRevealerAdded)) as Rts.CnC.Messages.Client.VisionRevealerAdded;
            //  Deserialize Id
            s.Read(out value.Id);
            //  Deserialize Position
            s.Read(out value.Position);
            //  Deserialize Radius
            s.Read(out value.Radius);

            return value;
        }
        
    }
}
