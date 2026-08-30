using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_SimuCloud_CreateGameFailure
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.SimuCloud.CreateGameFailure); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.SimuCloud.CreateGameFailure)obj;
            //  Serialize MapName
            s.Write(value.MapName);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.SimuCloud.CreateGameFailure)) as Rts.CnC.Messages.SimuCloud.CreateGameFailure;
            //  Deserialize MapName
            s.Read(out value.MapName);

            return value;
        }
        
    }
}
