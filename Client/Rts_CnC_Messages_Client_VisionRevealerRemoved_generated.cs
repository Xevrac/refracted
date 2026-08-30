using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_VisionRevealerRemoved
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.VisionRevealerRemoved); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.VisionRevealerRemoved)obj;
            //  Serialize Id
            s.Write(value.Id);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.VisionRevealerRemoved)) as Rts.CnC.Messages.Client.VisionRevealerRemoved;
            //  Deserialize Id
            s.Read(out value.Id);

            return value;
        }
        
    }
}
